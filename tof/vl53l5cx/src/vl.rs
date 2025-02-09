/*
* Presents one VL53L5CX sensor, with its activation line and unique I2C address.
*/
#[cfg(feature = "defmt")]
use defmt::debug;

use core::cell::RefCell;

use esp_hal::{
    gpio::Input,
    i2c::master::I2c,
    Blocking
};
#[cfg(feature = "flock")]
use esp_hal::gpio::Output;

use vl53l5cx_uld::{
    DEFAULT_I2C_ADDR,
    RangingConfig,
    Result,
    State_HP_Idle,
    VL53L5CX
};

use crate::{
    I2cAddr,
    uld_platform::Pl,
};

#[cfg(feature = "single")]
use crate::ranging::Ranging;
#[cfg(feature = "flock")]
use crate::ranging_flock::RangingFlock;

pub struct VL {
    uld: State_HP_Idle,   // initialized ULD level driver, with dedicated I2C address
}

impl VL {
    // tbd. make so that caller can give either 'I2cAddr' or a reference
    //
    pub fn new_and_setup(i2c_shared: &'static RefCell<I2c<'static, Blocking>>,
        i2c_addr: &I2cAddr
    ) -> Result<Self> {

        // Note: It seems the VL53L5CX doesn't retain its I2C address. Thus, we start each session
        //      by not only initializing the firmware (in '.init()') but also from the default I2C
        //      address. tbd. CONFIRM!!
        //
        let pl = Pl::new(i2c_shared);

        let mut uld = VL53L5CX::new_with_ping(pl)?.init()?;

        let a = i2c_addr;
        if *a != DEFAULT_I2C_ADDR {
            debug!("!!!! calling set_i2c_address: {}", a);
            uld.set_i2c_address(a)?;     // tbd. '.as_8bit()' if public
        }
        debug!("Board now reachable as: {}", i2c_addr);

        Ok(Self{
            uld,
        })
    }

    /*
    * Start ranging on a single board, with an 'INT' pin wired.
    */
    #[cfg(feature = "single")]
    pub fn start_ranging<const DIM: usize>(self, cfg: &RangingConfig<DIM>, pinINT: Input<'static>) -> Result<Ranging<DIM>> {
        Ranging::start(self, cfg, pinINT)
    }

    /*
    * A consuming method, used when moving to "Ranging" state.
    */
    pub(crate) fn into_uld(self) -> State_HP_Idle {
        self.uld
    }

    pub(crate) fn recreate(uld: State_HP_Idle) -> Self {
        Self { uld }
    }

    #[cfg(feature = "flock")]
    pub fn new_flock<const BOARDS: usize>(
        LPns: [Output;BOARDS],
        i2c_shared: &'static RefCell<I2c<'static, Blocking>>,
        i2c_addr_gen: impl Fn(usize) -> I2cAddr
    ) -> Result<[Self;BOARDS]> {
        fn array_try_map_mut_enumerated<A,B, const N: usize>(mut aa: [A;N], f: impl FnMut((usize,&mut A)) -> Result<B>) -> Result<[B;N]> {
            use arrayvec::ArrayVec;
            let bs_av = aa.iter_mut().enumerate().map(f)
                .collect::<Result<ArrayVec<B,N>>>();

            bs_av.map(|x| x.into_inner().ok().unwrap())
        }

        let tmp: Result<[VL;BOARDS]> = array_try_map_mut_enumerated(LPns, #[allow(non_snake_case)] |(i,LPn)| {
            LPn.set_high();     // enable this chip and leave it on

            let i2c_addr = i2c_addr_gen(i);
            debug!("I2C ADDR: {} -> {}", i, i2c_addr);   // TEMP
            let vl = VL::new_and_setup(i2c_shared, &i2c_addr)?;

            debug!("Init of board {} succeeded", i);
            Ok(vl)
        });
        tmp
    }
}

/*
* For multiple boards, we can extend the slice itself; this is really handy!
*
* Note: Ranging for a single board is done differently than for multiple, because there are
*       differences. The single board case doesn't need to suffer from unneeded complexity.
*/
#[cfg(feature = "flock")]
pub trait VLsExt<const N: usize, const DIM: usize> {
    fn start_ranging(self, cfg: &RangingConfig<DIM>, pinINT: Input<'static>) -> Result<RangingFlock<N,DIM>>;
}

#[cfg(feature = "flock")]
impl<const N: usize, const DIM: usize> VLsExt<N,DIM> for [VL;N] {
    fn start_ranging(self, cfg: &RangingConfig<DIM>, pinINT: Input<'static>) -> Result<RangingFlock<N,DIM>> {
        RangingFlock::start(self, cfg, pinINT)
    }
    /***
    <<
        Trait `FromIterator<Result<State_Ranging<{ DIM }>, Error>>` is not implemented for `[State_Ranging<{ DIM }>; N]` [E0277]
    <<
    let tmp = self.into_iter().map(|x|
        x.into_uld().start_ranging(cfg)
    ).collect::<[State_Ranging<DIM>;N]>();
    ...
    ***/
}
