#![allow(dead_code)]

use core::ffi::c_void;

use defmt::info;

use crate::sw_dsp::chapro::{
    _cc, cha_agc_channel, cha_agc_input, cha_agc_output, cha_agc_prepare, cha_firfb_analyze,
    cha_firfb_prepare, cha_firfb_synthesize, CHA_DSL, CHA_WDRC, NPTR,
};

#[repr(C)]
#[derive(Debug)]
pub struct FirFilterBank {
    sample_rate: f64, // rate or fs
    chunk_size: i32,  // cs
    cp: [*mut c_void; NPTR as usize],
    dsl: CHA_DSL,
    agc: CHA_WDRC,
    is_prepared: bool,
}

impl FirFilterBank {
    pub fn prepare(&mut self) {
        let cp = &mut self.cp as *mut _ as *mut *mut c_void;
        let sr = self.sample_rate;
        let cs = self.chunk_size;
        let nc = self.dsl.nchannel;
        let cf = &mut self.dsl.cross_freq as *mut _;
        let nw = self.agc.nw;
        let wt = self.agc.wt;

        // filterbank prepare
        let err = unsafe { cha_firfb_prepare(cp, cf, nc, sr, nw, wt, cs) };
        info!("cha_firfb_prepare err: {}", err);

        // automatic gain control prepare
        let err = unsafe { cha_agc_prepare(cp, &mut self.dsl as *mut _, &mut self.agc as *mut _) };
        info!("cha_agc_prepare err: {}", err);

        self.is_prepared = true;
    }

    pub fn process(&mut self, input: &mut [f32], output: &mut [f32]) {
        assert!(self.is_prepared);
        assert_eq!(input.len(), output.len());
        assert_eq!(input.len(), self.chunk_size as usize);

        let cp = &mut self.cp as *mut _ as *mut *mut c_void;
        let x = input as *mut _ as *mut f32;
        let y = output as *mut _ as *mut f32;
        let z = &mut self.cp[_cc as usize] as *mut _ as *mut *mut f32; // CHA_CP
        let z = unsafe { *z };
        let cs = self.chunk_size;

        unsafe {
            cha_agc_input(cp, x, x, cs);
            cha_firfb_analyze(cp, x, z, cs);
            cha_agc_channel(cp, z, z, cs);
            cha_firfb_synthesize(cp, z, y, cs);
            cha_agc_output(cp, y, y, cs);
        }
    }
}

impl Default for FirFilterBank {
    fn default() -> Self {
        // compressor config - desired sensation level
        static DSL: CHA_DSL = CHA_DSL {
            attack: 5.0,
            release: 50.0,
            maxdB: 119.0,
            ear: 0,
            nchannel: 8,
            cross_freq: [
                317.1666, 502.9734, 797.6319, 1264.9, 2005.9, 3181.1, 5044.7, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0,
            ],
            bolt: [
                -13.5942, -16.5909, -3.7978, 6.6176, 11.3050, 23.7183, 35.8586, 37.3885, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ],
            cr: [
                0.7, 0.9, 1.0, 1.1, 1.2, 1.4, 1.6, 1.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ],
            tk: [
                32.2, 26.5, 26.7, 26.7, 29.8, 33.6, 34.3, 32.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0,
            ],
            tkgain: [
                78.7667, 88.2, 90.7, 92.8333, 98.2, 103.3, 101.9, 99.8, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0,
            ],
        };

        // compressor config - wide dynamic range compressor (aka automatic gain control)
        static AGC: CHA_WDRC = CHA_WDRC {
            attack: 1.0,
            release: 50.0,
            fs: 24000.0,
            //fs: 50000.0,
            maxdB: 119.0,
            tkgain: 0.0,
            tk: 105.0,
            cr: 10.0,
            bolt: 105.0,
            td: 0.0,
            nw: 256, // window size
            nz: 0,
            wt: 0, // window type: 0=Hamming, 1=Blackman
        };

        static mut CP: [*mut c_void; NPTR as usize] = [0 as *mut c_void; NPTR as usize];

        Self {
            sample_rate: 24000.0,
            // sample_rate: 50000.0,
            chunk_size: 32, // num samples
            cp: unsafe { CP },
            dsl: DSL,
            agc: AGC,
            is_prepared: false,
        }
    }
}
