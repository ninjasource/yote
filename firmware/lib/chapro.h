#ifdef __cplusplus
extern "C"
{
#endif
// chapro.h - function prototypes for CHA common functions
#ifndef CHAPRO_H
#define CHAPRO_H

#ifdef WIN32
#include <io.h>
#include <Windows.h>
#endif

#if !defined(_MSC_VER) || (_MSC_VER > 1500)
#include <stdint.h>
#else
    typedef short int16_t;
    typedef long int32_t;
    typedef unsigned long uint32_t;
#endif

#ifdef DLL
#define FUNC(type) __declspec(dllexport) type _stdcall
#else
#define FUNC(type) type
#endif

#define NPTR 64
#define NVAR 32
#define CHA_IVAR ((int *)cp[_ivar])
#define CHA_DVAR ((double *)cp[_dvar])
#define CHA_CB ((float *)cp[_cc])

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif
#ifndef M_LN2
#define M_LN2 0.693147180559945309417
#endif
#ifndef M_SQRT1_2
#define M_SQRT1_2 0.707106781186547524401
#endif
#ifndef M_SQRT2
#define M_SQRT2 1.41421356237309504880
#endif
#ifdef MGW
#undef WIN32
#endif
#ifndef WIN32
#define _hypot hypot
#define _strdup strdup
#define __inline inline
#endif

#define fmin(x, y) ((x < y) ? (x) : (y))
#define fmove(x, y, n) memmove(x, y, (n) * sizeof(float))
#define fcopy(x, y, n) memcpy(x, y, (n) * sizeof(float))
#define fzero(x, n) memset(x, 0, (n) * sizeof(float))
#define dcopy(x, y, n) memcpy(x, y, (n) * sizeof(double))
#define dzero(x, n) memset(x, 0, (n) * sizeof(double))

#ifndef ARDUINO
#define round(x) ((int)floorf((float)(x) + 0.5f))
#endif

#ifndef log2
#define log2(x) (logf(x) / M_LN2)
#endif

#define db1(x) (logf(x) * 4.342944819032518f)   //   db1(x)=(log(x)*10/log(10))
#define db2(x) (logf(x) * 8.685889638065035f)   //   db2(x)=(log(x)*20/log(10))
#define undb1(x) expf((x) * 0.230258509299405f) // undb1(x)=exp((x)*log(10)/10)
#define undb2(x) expf((x) * 0.115129254649702f) // undb2(x)=exp((x)*log(10)/20)

    typedef uint32_t CHA_DATA;
    typedef void **CHA_PTR;

    /*****************************************************/

    // CHAPRO state

    typedef struct
    {
        int32_t arsiz; // total size of data (bytes)
        int32_t ptsiz; // number of data pointers
        void **cp;     // array of data pointers
        void *data;    // pointer to data
        double sr;     // sampling rate (kHz)
        int32_t cs;    // chunk size
        int32_t type;  // type of state
        int32_t entry; // entry point index
        void *rprt;    // pointer to report function
        void *proc;    // pointer to process function
    } CHA_STA;

    /*****************************************************/

    // DSL prescription

#define DSL_MXCH 32 // maximum number of channels

    typedef struct
    {
        double attack;               // attack time (ms)
        double release;              // release time (ms)
        double maxdB;                // maximum signal (dB SPL)
        int32_t ear;                 // 0=left, 1=right
        int32_t nchannel;            // number of channels
        double cross_freq[DSL_MXCH]; // cross frequencies (Hz)
        double tkgain[DSL_MXCH];     // compression-start gain
        double cr[DSL_MXCH];         // compression ratio
        double tk[DSL_MXCH];         // compression-start kneepoint
        double bolt[DSL_MXCH];       // broadband output limiting threshold
    } CHA_DSL;

    typedef struct
    {
        double attack;  // attack time (ms)
        double release; // release time (ms)
        double fs;      // sampling rate (Hz)
        double maxdB;   // maximum signal (dB SPL)
        double tkgain;  // compression-start gain
        double tk;      // compression-start kneepoint
        double cr;      // compression ratio
        double bolt;    // broadband output limiting threshold
        // processing parameters
        double td;  // target delay
        int32_t nz; // filter order
        int32_t nw; // window size
        int32_t wt; // window type: 0=Hamming, 1=Blackman
    } CHA_WDRC;

    /*****************************************************/

    // CLS prescription

#define CLS_MXCH 32 // maximum number of channels

    typedef struct
    {
        int32_t cm;           // compression mode
        int32_t nc;           // number of channels
        double fc[CLS_MXCH];  // center frequency
        double bw[CLS_MXCH];  // bandwith
        double Gcs[CLS_MXCH]; // gain at compression start
        double Gcm[CLS_MXCH]; // gain at compression middle
        double Gce[CLS_MXCH]; // gain at compression end
        double Gmx[CLS_MXCH]; // maximum gain
        double Lcs[CLS_MXCH]; // level at compression start
        double Lcm[CLS_MXCH]; // level at compression middle
        double Lce[CLS_MXCH]; // level at compression end
        double Lmx[CLS_MXCH]; // maximum output level
    } CHA_CLS;

    typedef struct
    {
        // processing
        double sr; // sampling rate (Hz)
        // FIR
        int32_t nz; // number of  poles & zeros
        int32_t nw; // window size
        int32_t wt; // window type: 0=Hamming, 1=Blackman
        // IIR
        double fd;  // estimated filter delay (ms)
        double gd;  // target delay (ms)
        double gn;  // flat gain (dB)
        int32_t nm; // number of frequency bands below 1 kHz
        int32_t po; // number of bands per octave above 1 kHz
        int32_t no; // gammatone filter order
    } CHA_ICMP;

    /*****************************************************/

    typedef struct
    {
        // simulation parameters
        double fbg; // simulated-feedback gain
        // AFC parameters
        double rho;   // forgetting factor
        double eps;   // power threshold
        double mu;    // step size
        double alf;   // band-limit update
        int32_t afl;  // adaptive-filter length
        int32_t wfl;  // whiten-filter length
        int32_t pfl;  // pre-emphasis-filter length
        int32_t fbl;  // simulated-feedback length
        int32_t hdel; // output/input hardware delay
        int32_t pup;  // band-limit update period
        // feedback filter buffers
        float *efbp; // estimated-feedback buffer pointer
        float *sfbp; // simulated-feedback buffer pointer
        float *wfrp; // whiten-filter buffer pointer
        float *ffrp; // pre-emphasis-filter buffer pointer
        // quality metric buffers & parameters
        float *qm;     // quality-metric buffer pointer
        int32_t *iqmp; // quality-metric index pointer
        int32_t nqm;   // quality-metric buffer size
        int32_t iqm;   // quality-metric index
        int32_t sqm;   // save quality metric ?
        CHA_PTR pcp;   // previous CHA_PTR
    } CHA_AFC;

    typedef struct
    {
        int32_t cs;  // chunk size
        int32_t nw;  // window size (pow2)
        int32_t wt;  // window type: 0=Hamming, 1=Blackman
        int32_t nm;  // frequency-map size
        double sr;   // sampling rate (Hz)
        double lbf;  // compression-lower-bound frequency (Hz)
        double ubf;  // compression-upper-bound frequency (Hz)
        int32_t *mm; // frequency-map pointer
        float *g1;   // pre-map gain pointer
        float *g2;   // post-map gain pointer
    } CHA_NFC;

    typedef struct
    {
        int32_t cs;  // chunk size
        int32_t nw;  // window size (pow2)
        int32_t wt;  // window type: 0=Hamming, 1=Blackman
        int32_t nm;  // frequency-map size
        int32_t xr;  // expansion ratio
        int32_t hbw; // half-band-width of suppression
        double sr;   // sampling rate (Hz)
        double lbf;  // compression-lower-bound frequency (Hz)
        double ubf;  // compression-upper-bound frequency (Hz)
        double ref;  // SPL reference
        double Gmax; // maximum gain (dB)
        double Lmax; // maximum output level (dB SPL)
        double Lckp; // compression knee-point input level (dB SPL)
        double Lekp; // expansion knee-point input level (dB SPL)
        int32_t *mm; // frequency-map pointer
        float *g1;   // pre-map gain pointer
        float *g2;   // post-map gain pointer
        float *supp; // suppressive influence pointer
    } CHA_SHA;

    /*****************************************************/

    // CHA common functions

    FUNC(void *)
    cha_allocate(CHA_PTR, int, int, int);
    FUNC(void)
    cha_cleanup(CHA_PTR);
    FUNC(void)
    cha_chunk_size(CHA_PTR, int);
    FUNC(int)
    cha_data_gen(CHA_PTR, char *);
    FUNC(int)
    cha_data_save(CHA_PTR, char *);
    FUNC(int)
    cha_data_load(CHA_PTR, char *);
    FUNC(int)
    cha_state_save(CHA_PTR, CHA_STA *);
    FUNC(int)
    cha_state_copy(CHA_STA *, CHA_STA *);
    FUNC(int)
    cha_state_free(CHA_STA *);
    FUNC(int)
    cha_fft(float *, int);
    FUNC(int)
    cha_ifft(float *, int);
    FUNC(void)
    cha_fft_cr(float *, int);
    FUNC(void)
    cha_fft_rc(float *, int);
    FUNC(void)
    cha_prepare(CHA_PTR);
    FUNC(void)
    cha_scale(float *, int, float);
    FUNC(char *)
    cha_version(void);

    /*****************************************************/

    // firfb module

    FUNC(int)
    cha_firfb_prepare(CHA_PTR, double *, int, double, int, int, int);
    FUNC(void)
    cha_firfb_analyze(CHA_PTR, float *, float *, int);
    FUNC(void)
    cha_firfb_synthesize(CHA_PTR, float *, float *, int);

    // iirfb module

    FUNC(int)
    cha_iirfb_design(float *, float *, float *, int *, double *, int, int, double, double);
    FUNC(int)
    cha_iirfb_prepare(CHA_PTR, float *, float *, float *, int *, int, int, double, int);
    FUNC(void)
    cha_iirfb_analyze(CHA_PTR, float *, float *, int);
    FUNC(void)
    cha_iirfb_synthesize(CHA_PTR, float *, float *, int);

    // cfirfb module

    FUNC(int)
    cha_cfirfb_prepare(CHA_PTR, double *, int, double, int, int, int);
    FUNC(void)
    cha_cfirfb_analyze(CHA_PTR, float *, float *, int);
    FUNC(void)
    cha_cfirfb_synthesize(CHA_PTR, float *, float *, int);

    // ciirfb module

    FUNC(int)
    cha_ciirfb_design(float *, float *, float *, int *, int, double *, double *, double, double);
    FUNC(int)
    cha_ciirfb_prepare(CHA_PTR, float *, float *, float *, int *, int, int, double, int);
    FUNC(void)
    cha_ciirfb_analyze(CHA_PTR, float *, float *, int);
    FUNC(void)
    cha_ciirfb_synthesize(CHA_PTR, float *, float *, int);

    // double ciirfb module

    FUNC(int)
    cha_dciirfb_prepare(CHA_PTR, float *, float *, float *, int *, int, int, double, int);
    FUNC(void)
    cha_dciirfb_analyze(CHA_PTR, float *, float *, int);
    FUNC(void)
    cha_dciirfb_synthesize(CHA_PTR, float *, float *, int);

    /*****************************************************/

    // AGC compressor module

    FUNC(int)
    cha_agc_prepare(CHA_PTR, CHA_DSL *, CHA_WDRC *);
    FUNC(void)
    cha_agc_input(CHA_PTR, float *, float *, int);
    FUNC(void)
    cha_agc_channel(CHA_PTR, float *, float *, int);
    FUNC(void)
    cha_agc_output(CHA_PTR, float *, float *, int);

    // instantanous compressor module

    FUNC(int)
    cha_icmp_prepare(CHA_PTR, CHA_CLS *, double, double, int);
    FUNC(void)
    cha_icmp_process(CHA_PTR, float *, float *, int);

    /*****************************************************/

    // feedback module

    FUNC(int)
    cha_afc_prepare(CHA_PTR, CHA_AFC *);
    FUNC(int)
    cha_afc_filters(CHA_PTR, CHA_AFC *);
    FUNC(void)
    cha_afc_input(CHA_PTR, float *, float *, int);
    FUNC(void)
    cha_afc_output(CHA_PTR, float *, int);

    /*****************************************************/

    // frequency-compression module

    FUNC(int)
    cha_nfc_prepare(CHA_PTR, CHA_NFC *);
    FUNC(void)
    cha_nfc_process(CHA_PTR, float *, float *, int);

    /*****************************************************/

    // suppression module

    FUNC(int)
    cha_sha_prepare(CHA_PTR, CHA_SHA *);
    FUNC(void)
    cha_sha_process(CHA_PTR, float *, float *, int);

    /*****************************************************/

    // global pointer indices

#define _size 0
#define _ivar 1
#define _dvar 2
#define _cc 3

#define _offset 4

    //----------------------------------------

    // fir & cfir pointer indices

#define _ffhh _offset + 1
#define _ffxx _offset + 2
#define _ffyy _offset + 3
#define _ffzz _offset + 4

    // fir & cfir pointer indices

#define _ffhh _offset + 1
#define _ffxx _offset + 2
#define _ffyy _offset + 3
#define _ffzz _offset + 4

    // iir pointer indices

#define _bb _offset + 1
#define _aa _offset + 2
#define _zz _offset + 3
#define _yd _offset + 4
#define _xx _offset + 5
#define _dd _offset + 6

    // ciir pointer indices

#define _dn _offset + 1
#define _ydr _offset + 2
#define _br _offset + 3
#define _ar _offset + 4
#define _zr _offset + 5
#define _zg _offset + 6

    //----------------------------------------

    // agc pointer indices

#define _gctk _offset + 7
#define _gccr _offset + 8
#define _gctkgn _offset + 9
#define _gcbolt _offset + 10
#define _gcppk _offset + 11
#define _xpk _offset + 12
#define _ppk _offset + 13

    // icmp pointer indices

#define _dsm _offset + 7
#define _dso _offset + 8
#define _c1 _offset + 9
#define _c2 _offset + 10
#define _fc _offset + 11
#define _bw _offset + 12
#define _Lcs _offset + 13
#define _Lcm _offset + 14
#define _Lce _offset + 15
#define _Lmx _offset + 16
#define _Gcs _offset + 17
#define _Gcm _offset + 18
#define _Gce _offset + 19
#define _Gmx _offset + 20
#define _Gmn _offset + 21
#define _Lfb _offset + 22
#define _Gsup _offset + 23
#define _Gpre _offset + 24
#define _gsup _offset + 25
#define _ginc _offset + 26
#define _zdr _offset + 27

    //----------------------------------------

    // afc pointer indices

#define _rng0 _offset + 28
#define _rng1 _offset + 29
#define _rng2 _offset + 30
#define _rng3 _offset + 31
#define _efbp _offset + 32
#define _sfbp _offset + 33
#define _wfrp _offset + 34
#define _ffrp _offset + 35
#define _qm _offset + 36
#define _iqmp _offset + 37

    // nfc pointer indices

#define _nfc_mm _offset + 38
#define _nfc_ww _offset + 39
#define _nfc_xx _offset + 40
#define _nfc_XX _offset + 41
#define _nfc_yy _offset + 42
#define _nfc_YY _offset + 43
#define _nfc_g1 _offset + 44
#define _nfc_g2 _offset + 45

    // sha pointer indices

#define _sha_mm _offset + 38
#define _sha_ww _offset + 39
#define _sha_xx _offset + 40
#define _sha_XX _offset + 41
#define _sha_yy _offset + 42
#define _sha_YY _offset + 43
#define _sha_g1 _offset + 44
#define _sha_g2 _offset + 45
#define _sha_SS _offset + 46
#define _sha_AA _offset + 47
#define _sha_II _offset + 48
#define _sha_JJ _offset + 49

    /*****************************************************/

    // global integer variable indices

#define _cs 0
#define _nc 1

    //----------------------------------------

    // iir integer variable indices

#define _op 2
#define _nn 3
#define _nw 4

    // ciir integer variable indices

#define _ns 2
#define _cm 3

    //----------------------------------------

    // icmp integer variable indices

#define _cm 3
#define _nw 4
#define _ds 5

    //----------------------------------------

    // afc integer variable indices

#define _rsz 6
#define _afl 7
#define _fbl 8
#define _nqm 9
#define _wfl 10
#define _pfl 11
#define _mxl 12
#define _in1 13
#define _in2 14
#define _rhd 15
#define _hdel 16
#define _pup 17
#define _puc 18
#define _rtl 19

    // nfc integer variable indices

#define _nfc_nw 20  // window size
#define _nfc_nm 21  // frequency-map size
#define _nfc_wt 22  // window window type
#define _nfc_ncs 23 // chunks-per-shift number
#define _nfc_ics 24 // chunks-per-shift count

    // sha integer variable indices

#define _sha_nw 20  // window size
#define _sha_nm 21  // frequency-map size
#define _sha_wt 22  // window window type
#define _sha_ncs 23 // chunks-per-shift number
#define _sha_ics 24 // chunks-per-shift count
#define _sha_xr 25  // expansion ratio
#define _sha_hbw 26 // half-band-width of suppression

    /*****************************************************/

    // global double variable indices

#define _fs 0

    //----------------------------------------

    // agc double variable indices

#define _alfa 1
#define _beta 2
#define _mxdb 3
#define _tkgn 4
#define _tk 5
#define _cr 6
#define _bolt 7
#define _gcalfa 8
#define _gcbeta 9

    // icmp double variable indices

#define _lrpk 1

    //----------------------------------------

    // afc double variable indices

#define _mu 10
#define _rho 11
#define _eps 12
#define _alf 13
#define _fbm 15
#define _pwr 16

    // sha double variable indices

#define _sha_g0 17
#define _sha_a1 18
#define _sha_a2 19
#define _sha_a3 20
#define _sha_gg 21

    /*****************************************************/

#endif /* CHAPRO_H */
#ifdef __cplusplus
} // extern "C"
#endif
