//
//   (C) Copyright 2017 CEA LIST. All Rights Reserved.
//   Contributor(s): Thibaud Tortech & Sergiu Carpov
//
//   This software is governed by the CeCILL-C license under French law and
//   abiding by the rules of distribution of free software.  You can  use,
//   modify and/ or redistribute the software under the terms of the CeCILL-C
//   license as circulated by CEA, CNRS and INRIA at the following URL
//   "http://www.cecill.info".
//
//   As a counterpart to the access to the source code and  rights to copy,
//   modify and redistribute granted by the license, users are provided only
//   with a limited warranty  and the software's author,  the holder of the
//   economic rights,  and the successive licensors  have only  limited
//   liability.
//
//   The fact that you are presently reading this means that you have had
//   knowledge of the CeCILL-C license and that you accept its terms.
//



const CHI2DF3_UB: f64 = 35.5;
const CHI2DF3_SF_VAL: [f64; 101] = [1.0, 0.9493731984407948, 0.8708493656619922, 0.7855292225296696, 0.7008533895214548, 0.6203902677759068, 0.5458674418310094, 0.4780080195886126, 0.4169574644339232, 0.3625260637072188, 0.3143349537389201, 0.2719068273779466, 0.2347229806494608, 0.2022590759013261, 0.1740071183759556, 0.1494883800946708, 0.128260350587543, 0.1099197600921883, 0.0941030530990517, 0.0804852479024174, 0.06877781936579971, 0.05872603787607948, 0.05010605635000589, 0.04272193908492043, 0.03640275792210043, 0.03099983364353439, 0.02638416759349314, 0.02244408592209809, 0.01908310357827561, 0.01621800509952656, 0.01377713283320039, 0.01169886938761112, 0.009930299068017611, 0.008426032239327591, 0.007147176570494977, 0.006060439666154512, 0.005137348473806351, 0.004353571925231854, 0.003688334428264771, 0.003123909000977183, 0.0026451799892456, 0.002239266401575878, 0.001895197914809837, 0.001603636542024127, 0.001356637806415451, 0.001147446032973333, 0.0009703190565314075, 0.0008203782551821963, 0.0006934803577347865, 0.0005861079489783587, 0.0004952760131371799, 0.0004184522200547148, 0.0003534889760156947, 0.0002985655370103923, 0.0002521387215426875, 0.0002129009672179402, 0.0001797446543155813, 0.0001517317739311227, 0.0001280681512434525, 0.0001080815488398237, 9.120307329149904e-05, 7.695139249275962e-05, 6.491934355737452e-05, 5.476257296878188e-05, 4.618990364643265e-05, 3.895516887352926e-05, 3.285029171564022e-05, 2.769942158327894e-05, 2.335396776399116e-05, 1.968839376840324e-05, 1.659665680040802e-05, 1.398919409024176e-05, 1.179037266364586e-05, 9.936331740578788e-06, 8.373157688565748e-06, 7.055340577413872e-06, 5.944469132008688e-06, 5.008127461925064e-06, 4.218962534576839e-06, 3.553896101036495e-06, 2.993458807083086e-06, 2.52122763439132e-06, 2.123350709982886e-06, 1.788145974389156e-06, 1.505762277026871e-06, 1.267893227717488e-06, 1.067535624255233e-06, 8.98785538381802e-07, 7.566662112330366e-07, 6.369828138143871e-07, 5.36199893405655e-07, 4.513379742632384e-07, 3.798863286256414e-07, 3.197293971384877e-07, 2.690847293728686e-07, 2.264506461117065e-07, 1.905621048402571e-07, 1.603534862914778e-07, 1.349272196490312e-07, 1.135273327594855e-07, 9.551715624143771e-08];
const CHI2DF3_CONST: f64 = 0.5641895835477562;
const CHI2DF3_INTEG_STEPS: i32 = 1000;

/**
 * @brief Chi-square 3-DOF probability distribution function
 */
fn chi2df3_pdf(x: f64) -> f64 {
    use core::intrinsics::{sqrtf64,expf64};
    let a: f64 = x/2.0;
    unsafe {sqrtf64(a) * expf64(-a) * CHI2DF3_CONST}
}

/**
 * @brief Chi-square 3-DOF integral over [a;b]
 */
fn chi2df3_sf_integ(a: f64, b: f64) -> f64 {
    let mut t = a;
    let step = (b-a) / CHI2DF3_INTEG_STEPS as f64;
    let mut integ_sum = (chi2df3_pdf(a) + chi2df3_pdf(b)) / 2.0;
    for _ in 1..CHI2DF3_INTEG_STEPS {
        t += step;
        integ_sum += chi2df3_pdf(t);
    }
    integ_sum * step
}

/**
 * @brief Conditional selection 
 * @return a if cond==True else b
 */
fn cond_select(cond: bool, a: f64, b: f64) -> f64 {
    let c = cond as i32 as f64;
    c*(a-b)+b
}

/**
 * @brief Chi-square 3-DOF survival function 
 */
pub fn chi2df3_sf(x: f64) -> f64 {
    let delta = CHI2DF3_UB / (CHI2DF3_SF_VAL.len() as f64 - 1.0);

    let mut a = 0.0;
    let mut b = 0.0;
    let mut left_chi2val = 0.0;
    let mut right_chi2val = 0.0;
    for i in 0..(CHI2DF3_SF_VAL.len()-1) {
        let d1 = delta*i as f64;
        let d2 = delta*(i+1) as f64;
        let c = ((d1 <= x)&(x < d2)) as i8 as f64;

        a += c*d1;
        b += c*d2;
        
        left_chi2val += c*CHI2DF3_SF_VAL[i];
        right_chi2val += c*CHI2DF3_SF_VAL[i+1];
    }

    let c = (x-a)<(b-x);

    a = cond_select(c, a, x);
    b = cond_select(c, x, b);
    let integ_sum = chi2df3_sf_integ(a,b);

    left_chi2val -= integ_sum;
    right_chi2val += integ_sum;
    let res = cond_select(c, left_chi2val, right_chi2val);

    let in_limit = (x<=CHI2DF3_UB) as i8 as f64;
    res * in_limit
}

const PSEUDO: f64 = 1.0e-15;

/**
 * @brief Compute chi-square statistic p-value
 */
pub fn chisquare_stat(n: f64, n1: f64, n2: f64, n1g: f64, n2g: f64) -> f64 {
    let ng = n1g+n2g;

    let t = n1*n2g - n2*n1g;
    let chi2 = (t*t*n) / (n1*n2*ng*(n-ng+PSEUDO));

    chi2
    // af = (n1g+n2g)/n
}


// pub fn chisquare_stats(n: f64, n1: f64, n2: f64, n1g: f64, n2g: f64) -> (f64,f64,f64) {
//     // let n = n1+n2;

//     let ng = n1g+n2g;

//     let t = n1*n2g - n2*n1g;
//     let chi2 = (t*t*n) / (n1*n2*ng*(n-ng+PSEUDO));
//     let af = (n1g+n2g)/n;

//     (chi2df3_sf(chi2), af, chi2)
// }

