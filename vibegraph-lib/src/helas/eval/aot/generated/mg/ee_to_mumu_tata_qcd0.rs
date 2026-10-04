// chunk: 0. Rendered from the helicity-expanded program of `ee_to_mumu_tata_qcd0` (1756 instructions, 1716 calls, 1 function(s), slots per class [18, 801, 104, 0, 31, 19]); regenerate, do not edit.
pub(super) fn bind_ee_to_mumu_tata_qcd0<F: Real>() -> Box<dyn MgRun<F>> {
    bound::<F, 801, 104, 0, 31, 19>(mg_ee_to_mumu_tata_qcd0::<F>)
}
#[inline(never)]
fn mg_ee_to_mumu_tata_qcd0<F: Real>(a: &mut MgArenas<F, 801, 104, 0, 31, 19>, mo: &[LorentzVector<F>], cc: &[C<F>], cr: &[F], mm: &[LorentzVector<F>]) {
    let mo: &[LorentzVector<F>; 6] = mo.try_into().expect("external momenta");
    let cc: &[C<F>; 7] = cc.try_into().expect("complex pool");
    let cr: &[F; 20] = cr.try_into().expect("real pool");
    let mm: &[LorentzVector<F>; 20] = mm.try_into().expect("momentum pool");
    k::ext_fin(&mut a.i[0], &mo[3], 1, 2, Charge::Antiparticle, false, &cr[0]);
    k::ext_fin(&mut a.i[1], &mo[5], -1, 2, Charge::Antiparticle, false, &cr[1]);
    k::ext_fin(&mut a.i[2], &mo[1], 1, 2, Charge::Particle, true, &cr[9]);
    k::ext_fin(&mut a.i[3], &mo[5], 1, 2, Charge::Antiparticle, false, &cr[1]);
    k::ext_fin(&mut a.i[4], &mo[3], -1, 2, Charge::Antiparticle, false, &cr[0]);
    k::ext_fin(&mut a.i[5], &mo[1], -1, 2, Charge::Particle, true, &cr[9]);
    k::ext_fout(&mut a.o[0], &mo[4], -1, 2, Charge::Particle, false, &cr[2]);
    k::ext_fout(&mut a.o[1], &mo[2], -1, 2, Charge::Particle, false, &cr[6]);
    k::ext_fout(&mut a.o[2], &mo[0], -1, 2, Charge::Antiparticle, true, &cr[8]);
    k::ext_fout(&mut a.o[3], &mo[4], 1, 2, Charge::Particle, false, &cr[2]);
    k::ext_fout(&mut a.o[4], &mo[2], 1, 2, Charge::Particle, false, &cr[6]);
    k::ext_fout(&mut a.o[5], &mo[0], 1, 2, Charge::Antiparticle, true, &cr[8]);
    k::gamma_vout(&mut a.v[0], &a.o[0], &a.i[1], false);
    k::gamma_vout(&mut a.v[1], &a.o[2], &a.i[2], true);
    k::gamma_vout(&mut a.v[2], &a.o[1], &a.i[0], false);
    k::gamma_vout(&mut a.v[3], &a.o[0], &a.i[3], false);
    k::gamma_vout(&mut a.v[4], &a.o[3], &a.i[1], false);
    k::gamma_vout(&mut a.v[5], &a.o[3], &a.i[3], false);
    k::gamma_vout(&mut a.v[6], &a.o[4], &a.i[4], false);
    k::gamma_vout(&mut a.v[7], &a.o[5], &a.i[5], true);
    k::ffv_vout(&mut a.v[8], &a.o[0], &a.i[1], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[9], &a.o[2], &a.i[2], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[10], &a.o[1], &a.i[0], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[11], &a.o[0], &a.i[3], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[12], &a.o[3], &a.i[1], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[13], &a.o[3], &a.i[3], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[14], &a.o[4], &a.i[4], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[15], &a.o[5], &a.i[5], &cc[5], &cc[6], true);
    k::scalar_bilinear(&mut a.s[5], &a.o[0], &a.i[1], Chirality::Left);
    k::scalar_bilinear(&mut a.s[6], &a.o[0], &a.i[1], Chirality::Right);
    k::scalar_bilinear(&mut a.s[7], &a.o[0], &a.i[3], Chirality::Left);
    k::scalar_bilinear(&mut a.s[8], &a.o[0], &a.i[3], Chirality::Right);
    k::scalar_bilinear(&mut a.s[9], &a.o[3], &a.i[1], Chirality::Left);
    k::scalar_bilinear(&mut a.s[10], &a.o[3], &a.i[1], Chirality::Right);
    k::scalar_bilinear(&mut a.s[11], &a.o[3], &a.i[3], Chirality::Left);
    k::scalar_bilinear(&mut a.s[12], &a.o[3], &a.i[3], Chirality::Right);
    { let (l, d, _) = sp(&mut a.v, 16); k::propagate_vector(d, &l[8], &mm[4], &cr[11], &cr[12]); }
    { let (_, d, h) = sp(&mut a.v, 8); k::propagate_vector(d, &h[0], &mm[9], &cr[11], &cr[12]); }
    { let (_, d, h) = sp(&mut a.v, 9); k::propagate_vector(d, &h[0], &mm[13], &cr[11], &cr[12]); }
    { let (_, d, h) = sp(&mut a.v, 10); k::propagate_vector(d, &h[0], &mm[4], &cr[11], &cr[12]); }
    { let (_, d, h) = sp(&mut a.v, 11); k::propagate_vector(d, &h[0], &mm[4], &cr[11], &cr[12]); }
    { let (_, d, h) = sp(&mut a.v, 12); k::propagate_vector(d, &h[0], &mm[4], &cr[11], &cr[12]); }
    { let (_, d, h) = sp(&mut a.v, 13); k::propagate_vector(d, &h[0], &mm[13], &cr[11], &cr[12]); }
    { let (_, d, h) = sp(&mut a.v, 14); k::propagate_vector(d, &h[0], &mm[9], &cr[11], &cr[12]); }
    { let (l, d, _) = sp(&mut a.s, 13); k::mul(d, &l[5], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 5); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 6); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 7); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 8); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 9); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 10); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 11); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.v, 15); k::mul(d, &l[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.v, 0); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.v, 1); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.v, 2); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.v, 3); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.v, 4); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.v, 5); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.v, 6); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, h) = sp(&mut a.s, 12); k::add(d, &[&h[0], &l[5]]); }
    { let (_, d, h) = sp(&mut a.s, 5); k::add(d, &[&h[0], &h[1]]); }
    { let (_, d, h) = sp(&mut a.s, 7); k::add(d, &[&h[0], &h[1]]); }
    { let (_, d, h) = sp(&mut a.s, 9); k::add(d, &[&h[0], &h[1]]); }
    { let (_, d, h) = sp(&mut a.v, 7); k::mul(d, &h[7], &cc[0]); }
    { let (l, d, _) = sp(&mut a.v, 15); k::mul(d, &l[0], &cc[0]); }
    { let (_, d, h) = sp(&mut a.v, 0); k::mul(d, &h[0], &cc[0]); }
    { let (_, d, h) = sp(&mut a.v, 1); k::mul(d, &h[0], &cc[0]); }
    { let (_, d, h) = sp(&mut a.v, 2); k::mul(d, &h[0], &cc[0]); }
    { let (_, d, h) = sp(&mut a.v, 3); k::mul(d, &h[0], &cc[0]); }
    { let (_, d, h) = sp(&mut a.v, 4); k::mul(d, &h[0], &cc[0]); }
    { let (_, d, h) = sp(&mut a.v, 5); k::mul(d, &h[0], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 6); k::ffv_fin(d, &a.v[16], &l[0], &cc[5], &cc[6]); }
    { let (l, d, _) = sp(&mut a.i, 7); k::ffv_fin(d, &a.v[9], &l[1], &cc[5], &cc[6]); }
    { let (l, d, _) = sp(&mut a.i, 8); k::ffv_fin(d, &a.v[16], &l[2], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.i, 9); k::ffv_fin(d, &a.v[9], &l[2], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.i, 10); k::ffv_fin(d, &a.v[10], &l[0], &cc[5], &cc[6]); }
    { let (l, d, _) = sp(&mut a.i, 11); k::ffv_fin(d, &a.v[9], &l[3], &cc[5], &cc[6]); }
    { let (l, d, _) = sp(&mut a.i, 12); k::ffv_fin(d, &a.v[10], &l[2], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.i, 13); k::ffv_fin(d, &a.v[11], &l[0], &cc[5], &cc[6]); }
    { let (l, d, _) = sp(&mut a.i, 14); k::ffv_fin(d, &a.v[11], &l[2], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.i, 15); k::ffv_fin(d, &a.v[12], &l[0], &cc[5], &cc[6]); }
    { let (l, d, _) = sp(&mut a.i, 16); k::ffv_fin(d, &a.v[12], &l[2], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.i, 17); k::ffv_fin(d, &a.v[16], &l[4], &cc[5], &cc[6]); }
    { let (l, d, _) = sp(&mut a.i, 18); k::ffv_fin(d, &a.v[13], &l[1], &cc[5], &cc[6]); }
    { let (l, d, _) = sp(&mut a.i, 19); k::ffv_fin(d, &a.v[13], &l[2], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.i, 20); k::ffv_fin(d, &a.v[10], &l[4], &cc[5], &cc[6]); }
    { let (l, d, _) = sp(&mut a.i, 21); k::ffv_fin(d, &a.v[13], &l[3], &cc[5], &cc[6]); }
    { let (l, d, _) = sp(&mut a.i, 22); k::ffv_fin(d, &a.v[11], &l[4], &cc[5], &cc[6]); }
    { let (l, d, _) = sp(&mut a.i, 23); k::ffv_fin(d, &a.v[12], &l[4], &cc[5], &cc[6]); }
    { let (l, d, _) = sp(&mut a.i, 24); k::ffv_fin(d, &a.v[16], &l[5], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.i, 25); k::ffv_fin(d, &a.v[9], &l[5], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.i, 26); k::ffv_fin(d, &a.v[10], &l[5], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.i, 27); k::ffv_fin(d, &a.v[11], &l[5], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.i, 28); k::ffv_fin(d, &a.v[12], &l[5], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.i, 29); k::ffv_fin(d, &a.v[13], &l[5], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.o, 6); k::ffv_fout(d, &a.v[16], &l[1], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.o, 7); k::ffv_fout(d, &a.v[9], &l[0], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.o, 8); k::ffv_fout(d, &a.v[10], &l[1], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.o, 9); k::ffv_fout(d, &a.v[11], &l[1], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.o, 10); k::ffv_fout(d, &a.v[9], &l[3], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.o, 11); k::ffv_fout(d, &a.v[12], &l[1], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.o, 12); k::ffv_fout(d, &a.v[16], &l[4], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.o, 13); k::ffv_fout(d, &a.v[13], &l[0], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.o, 14); k::ffv_fout(d, &a.v[10], &l[4], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.o, 15); k::ffv_fout(d, &a.v[11], &l[4], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.o, 16); k::ffv_fout(d, &a.v[13], &l[3], &cc[6], &cc[5]); }
    { let (l, d, _) = sp(&mut a.o, 17); k::ffv_fout(d, &a.v[12], &l[4], &cc[6], &cc[5]); }
    k::metric(&mut a.s[11], &a.v[8], &a.v[9]);
    k::metric(&mut a.s[10], &a.v[8], &a.v[13]);
    k::metric(&mut a.s[8], &a.v[14], &a.v[9]);
    k::metric(&mut a.s[6], &a.v[14], &a.v[13]);
    { let (_, d, h) = sp(&mut a.v, 6); k::propagate_vector(d, &h[0], &mm[4], &cr[4], &cr[5]); }
    { let (_, d, h) = sp(&mut a.v, 7); k::propagate_vector(d, &h[7], &mm[9], &cr[4], &cr[5]); }
    { let (l, d, _) = sp(&mut a.v, 15); k::propagate_vector(d, &l[0], &mm[13], &cr[4], &cr[5]); }
    { let (_, d, h) = sp(&mut a.v, 0); k::propagate_vector(d, &h[0], &mm[4], &cr[4], &cr[5]); }
    { let (_, d, h) = sp(&mut a.v, 1); k::propagate_vector(d, &h[0], &mm[4], &cr[4], &cr[5]); }
    { let (_, d, h) = sp(&mut a.v, 2); k::propagate_vector(d, &h[0], &mm[4], &cr[4], &cr[5]); }
    { let (_, d, h) = sp(&mut a.v, 3); k::propagate_vector(d, &h[0], &mm[13], &cr[4], &cr[5]); }
    { let (_, d, h) = sp(&mut a.v, 4); k::propagate_vector(d, &h[0], &mm[9], &cr[4], &cr[5]); }
    { let (l, d, _) = sp(&mut a.i, 30); k::propagate_fin(d, &l[6], &mm[5], &cr[6], &cr[7]); }
    { let (_, d, h) = sp(&mut a.i, 6); k::propagate_fin(d, &h[0], &mm[14], &cr[2], &cr[14]); }
    { let (_, d, h) = sp(&mut a.i, 7); k::propagate_fin(d, &h[0], &mm[17], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 8); k::propagate_fin(d, &h[0], &mm[19], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 9); k::propagate_fin(d, &h[0], &mm[5], &cr[6], &cr[7]); }
    { let (_, d, h) = sp(&mut a.i, 10); k::propagate_fin(d, &h[0], &mm[14], &cr[2], &cr[14]); }
    { let (_, d, h) = sp(&mut a.i, 11); k::propagate_fin(d, &h[0], &mm[17], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 12); k::propagate_fin(d, &h[0], &mm[5], &cr[6], &cr[7]); }
    { let (_, d, h) = sp(&mut a.i, 13); k::propagate_fin(d, &h[0], &mm[17], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 14); k::propagate_fin(d, &h[0], &mm[5], &cr[6], &cr[7]); }
    { let (_, d, h) = sp(&mut a.i, 15); k::propagate_fin(d, &h[0], &mm[17], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 16); k::propagate_fin(d, &h[0], &mm[5], &cr[6], &cr[7]); }
    { let (_, d, h) = sp(&mut a.i, 17); k::propagate_fin(d, &h[0], &mm[14], &cr[2], &cr[14]); }
    { let (_, d, h) = sp(&mut a.i, 18); k::propagate_fin(d, &h[0], &mm[19], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 19); k::propagate_fin(d, &h[0], &mm[5], &cr[6], &cr[7]); }
    { let (_, d, h) = sp(&mut a.i, 20); k::propagate_fin(d, &h[0], &mm[14], &cr[2], &cr[14]); }
    { let (_, d, h) = sp(&mut a.i, 21); k::propagate_fin(d, &h[0], &mm[5], &cr[6], &cr[7]); }
    { let (_, d, h) = sp(&mut a.i, 22); k::propagate_fin(d, &h[0], &mm[5], &cr[6], &cr[7]); }
    { let (_, d, h) = sp(&mut a.i, 23); k::propagate_fin(d, &h[0], &mm[17], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 24); k::propagate_fin(d, &h[0], &mm[19], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 25); k::propagate_fin(d, &h[0], &mm[17], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 26); k::propagate_fin(d, &h[0], &mm[17], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 27); k::propagate_fin(d, &h[0], &mm[17], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 28); k::propagate_fin(d, &h[0], &mm[19], &cr[8], &cr[18]); }
    { let (l, d, _) = sp(&mut a.o, 18); k::propagate_fout(d, &l[6], &mm[12], &cr[0], &cr[13]); }
    { let (_, d, h) = sp(&mut a.o, 6); k::propagate_fout(d, &h[0], &mm[15], &cr[1], &cr[15]); }
    { let (_, d, h) = sp(&mut a.o, 7); k::propagate_fout(d, &h[0], &mm[12], &cr[0], &cr[13]); }
    { let (_, d, h) = sp(&mut a.o, 8); k::propagate_fout(d, &h[0], &mm[12], &cr[0], &cr[13]); }
    { let (_, d, h) = sp(&mut a.o, 9); k::propagate_fout(d, &h[0], &mm[15], &cr[1], &cr[15]); }
    { let (_, d, h) = sp(&mut a.o, 10); k::propagate_fout(d, &h[0], &mm[12], &cr[0], &cr[13]); }
    { let (_, d, h) = sp(&mut a.o, 11); k::propagate_fout(d, &h[0], &mm[12], &cr[0], &cr[13]); }
    { let (_, d, h) = sp(&mut a.o, 12); k::propagate_fout(d, &h[0], &mm[15], &cr[1], &cr[15]); }
    { let (_, d, h) = sp(&mut a.o, 13); k::propagate_fout(d, &h[0], &mm[12], &cr[0], &cr[13]); }
    { let (_, d, h) = sp(&mut a.o, 14); k::propagate_fout(d, &h[0], &mm[12], &cr[0], &cr[13]); }
    { let (_, d, h) = sp(&mut a.o, 15); k::propagate_fout(d, &h[0], &mm[15], &cr[1], &cr[15]); }
    { let (_, d, h) = sp(&mut a.o, 16); k::propagate_fout(d, &h[0], &mm[12], &cr[0], &cr[13]); }
    { let (l, d, _) = sp(&mut a.s, 13); k::mul(d, &cc[3], &l[12]); }
    { let (l, d, _) = sp(&mut a.s, 12); k::mul(d, &cc[3], &l[5]); }
    { let (_, d, h) = sp(&mut a.s, 5); k::mul(d, &cc[3], &h[1]); }
    { let (_, d, h) = sp(&mut a.s, 7); k::mul(d, &cc[3], &h[1]); }
    { let (_, d, h) = sp(&mut a.s, 9); k::propagate_scalar(d, &h[3], &mm[4], &cr[16], &cr[17]); }
    { let (l, d, _) = sp(&mut a.s, 13); k::propagate_scalar(d, &l[12], &mm[4], &cr[16], &cr[17]); }
    { let (l, d, _) = sp(&mut a.s, 12); k::propagate_scalar(d, &l[5], &mm[4], &cr[16], &cr[17]); }
    { let (_, d, h) = sp(&mut a.s, 5); k::propagate_scalar(d, &h[1], &mm[4], &cr[16], &cr[17]); }
    k::gamma_vout(&mut a.v[5], &a.o[1], &a.i[30], false);
    k::gamma_vout(&mut a.v[17], &a.o[18], &a.i[0], false);
    k::gamma_vout(&mut a.v[18], &a.o[0], &a.i[6], false);
    k::gamma_vout(&mut a.v[19], &a.o[6], &a.i[1], false);
    k::gamma_vout(&mut a.v[20], &a.o[2], &a.i[7], true);
    k::gamma_vout(&mut a.v[21], &a.o[2], &a.i[8], true);
    k::gamma_vout(&mut a.v[22], &a.o[1], &a.i[9], false);
    k::gamma_vout(&mut a.v[23], &a.o[7], &a.i[0], false);
    k::gamma_vout(&mut a.v[24], &a.o[0], &a.i[10], false);
    k::gamma_vout(&mut a.v[25], &a.o[6], &a.i[3], false);
    k::gamma_vout(&mut a.v[26], &a.o[2], &a.i[11], true);
    k::gamma_vout(&mut a.v[27], &a.o[1], &a.i[12], false);
    k::gamma_vout(&mut a.v[28], &a.o[8], &a.i[0], false);
    k::gamma_vout(&mut a.v[29], &a.o[3], &a.i[6], false);
    k::gamma_vout(&mut a.v[30], &a.o[9], &a.i[1], false);
    k::gamma_vout(&mut a.v[31], &a.o[2], &a.i[13], true);
    k::gamma_vout(&mut a.v[32], &a.o[1], &a.i[14], false);
    k::gamma_vout(&mut a.v[33], &a.o[10], &a.i[0], false);
    k::gamma_vout(&mut a.v[34], &a.o[3], &a.i[10], false);
    k::gamma_vout(&mut a.v[35], &a.o[9], &a.i[3], false);
    k::gamma_vout(&mut a.v[36], &a.o[2], &a.i[15], true);
    k::gamma_vout(&mut a.v[37], &a.o[4], &a.i[16], false);
    k::gamma_vout(&mut a.v[38], &a.o[11], &a.i[4], false);
    k::gamma_vout(&mut a.v[39], &a.o[0], &a.i[17], false);
    k::gamma_vout(&mut a.v[40], &a.o[12], &a.i[1], false);
    k::gamma_vout(&mut a.v[41], &a.o[2], &a.i[18], true);
    k::gamma_vout(&mut a.v[42], &a.o[4], &a.i[19], false);
    k::gamma_vout(&mut a.v[43], &a.o[13], &a.i[4], false);
    k::gamma_vout(&mut a.v[44], &a.o[0], &a.i[20], false);
    k::gamma_vout(&mut a.v[45], &a.o[12], &a.i[3], false);
    k::gamma_vout(&mut a.v[46], &a.o[4], &a.i[21], false);
    k::gamma_vout(&mut a.v[47], &a.o[14], &a.i[4], false);
    k::gamma_vout(&mut a.v[48], &a.o[3], &a.i[17], false);
    k::gamma_vout(&mut a.v[49], &a.o[15], &a.i[1], false);
    k::gamma_vout(&mut a.v[50], &a.o[4], &a.i[22], false);
    k::gamma_vout(&mut a.v[51], &a.o[16], &a.i[4], false);
    k::gamma_vout(&mut a.v[52], &a.o[3], &a.i[20], false);
    k::gamma_vout(&mut a.v[53], &a.o[15], &a.i[3], false);
    k::gamma_vout(&mut a.v[54], &a.o[5], &a.i[23], true);
    k::gamma_vout(&mut a.v[55], &a.o[5], &a.i[24], true);
    k::gamma_vout(&mut a.v[56], &a.o[5], &a.i[25], true);
    k::gamma_vout(&mut a.v[57], &a.o[5], &a.i[26], true);
    k::gamma_vout(&mut a.v[58], &a.o[5], &a.i[27], true);
    k::gamma_vout(&mut a.v[59], &a.o[5], &a.i[28], true);
    k::ffv_vout(&mut a.v[60], &a.o[1], &a.i[30], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[61], &a.o[18], &a.i[0], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[62], &a.o[0], &a.i[6], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[63], &a.o[6], &a.i[1], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[64], &a.o[2], &a.i[7], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[65], &a.o[2], &a.i[8], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[66], &a.o[1], &a.i[9], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[67], &a.o[7], &a.i[0], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[68], &a.o[0], &a.i[10], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[69], &a.o[6], &a.i[3], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[70], &a.o[2], &a.i[11], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[71], &a.o[1], &a.i[12], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[72], &a.o[8], &a.i[0], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[73], &a.o[3], &a.i[6], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[74], &a.o[9], &a.i[1], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[75], &a.o[2], &a.i[13], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[76], &a.o[1], &a.i[14], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[77], &a.o[10], &a.i[0], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[78], &a.o[3], &a.i[10], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[79], &a.o[9], &a.i[3], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[80], &a.o[2], &a.i[15], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[81], &a.o[4], &a.i[16], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[82], &a.o[11], &a.i[4], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[83], &a.o[0], &a.i[17], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[84], &a.o[12], &a.i[1], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[85], &a.o[2], &a.i[18], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[86], &a.o[4], &a.i[19], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[87], &a.o[13], &a.i[4], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[88], &a.o[0], &a.i[20], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[89], &a.o[12], &a.i[3], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[90], &a.o[4], &a.i[21], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[91], &a.o[14], &a.i[4], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[92], &a.o[3], &a.i[17], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[93], &a.o[15], &a.i[1], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[94], &a.o[4], &a.i[22], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[95], &a.o[16], &a.i[4], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[96], &a.o[3], &a.i[20], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[97], &a.o[15], &a.i[3], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[98], &a.o[5], &a.i[23], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[99], &a.o[5], &a.i[24], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[100], &a.o[5], &a.i[25], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[101], &a.o[5], &a.i[26], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[102], &a.o[5], &a.i[27], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[103], &a.o[5], &a.i[28], &cc[5], &cc[6], true);
    { let (l, d, _) = sp(&mut a.i, 28); k::off_shell_fin(d, &a.v[6], &l[0]); }
    { let (l, d, _) = sp(&mut a.i, 27); k::off_shell_fin(d, &a.v[15], &l[1]); }
    { let (l, d, _) = sp(&mut a.i, 26); k::off_shell_fin(d, &a.v[6], &l[2]); }
    { let (l, d, _) = sp(&mut a.i, 25); k::off_shell_fin(d, &a.v[15], &l[2]); }
    { let (l, d, _) = sp(&mut a.i, 24); k::off_shell_fin(d, &a.v[0], &l[0]); }
    { let (l, d, _) = sp(&mut a.i, 23); k::off_shell_fin(d, &a.v[15], &l[3]); }
    { let (l, d, _) = sp(&mut a.i, 20); k::off_shell_fin(d, &a.v[0], &l[2]); }
    { let (l, d, _) = sp(&mut a.i, 22); k::off_shell_fin(d, &a.v[1], &l[0]); }
    { let (l, d, _) = sp(&mut a.i, 17); k::off_shell_fin(d, &a.v[1], &l[2]); }
    { let (l, d, _) = sp(&mut a.i, 21); k::off_shell_fin(d, &a.v[2], &l[0]); }
    { let (l, d, _) = sp(&mut a.i, 19); k::off_shell_fin(d, &a.v[2], &l[2]); }
    { let (l, d, _) = sp(&mut a.i, 18); k::off_shell_fin(d, &a.v[6], &l[4]); }
    { let (l, d, _) = sp(&mut a.i, 16); k::off_shell_fin(d, &a.v[3], &l[1]); }
    { let (l, d, _) = sp(&mut a.i, 15); k::off_shell_fin(d, &a.v[3], &l[2]); }
    { let (_, d, h) = sp(&mut a.i, 2); k::off_shell_fin(d, &a.v[0], &h[1]); }
    { let (l, d, _) = sp(&mut a.i, 10); k::off_shell_fin(d, &a.v[3], &l[3]); }
    { let (l, d, _) = sp(&mut a.i, 14); k::off_shell_fin(d, &a.v[1], &l[4]); }
    { let (l, d, _) = sp(&mut a.i, 13); k::off_shell_fin(d, &a.v[2], &l[4]); }
    { let (l, d, _) = sp(&mut a.i, 6); k::off_shell_fin(d, &a.v[6], &l[5]); }
    { let (l, d, _) = sp(&mut a.i, 12); k::off_shell_fin(d, &a.v[15], &l[5]); }
    { let (l, d, _) = sp(&mut a.i, 11); k::off_shell_fin(d, &a.v[0], &l[5]); }
    { let (l, d, _) = sp(&mut a.i, 9); k::off_shell_fin(d, &a.v[1], &l[5]); }
    { let (l, d, _) = sp(&mut a.i, 8); k::off_shell_fin(d, &a.v[2], &l[5]); }
    { let (l, d, _) = sp(&mut a.i, 7); k::off_shell_fin(d, &a.v[3], &l[5]); }
    { let (l, d, _) = sp(&mut a.o, 15); k::off_shell_fout(d, &a.v[6], &l[1]); }
    { let (l, d, _) = sp(&mut a.o, 16); k::off_shell_fout(d, &a.v[15], &l[0]); }
    { let (l, d, _) = sp(&mut a.o, 14); k::off_shell_fout(d, &a.v[0], &l[1]); }
    { let (l, d, _) = sp(&mut a.o, 12); k::off_shell_fout(d, &a.v[1], &l[1]); }
    { let (l, d, _) = sp(&mut a.o, 13); k::off_shell_fout(d, &a.v[15], &l[3]); }
    { let (l, d, _) = sp(&mut a.o, 11); k::off_shell_fout(d, &a.v[2], &l[1]); }
    { let (l, d, _) = sp(&mut a.o, 9); k::off_shell_fout(d, &a.v[6], &l[4]); }
    { let (l, d, _) = sp(&mut a.o, 10); k::off_shell_fout(d, &a.v[3], &l[0]); }
    { let (l, d, _) = sp(&mut a.o, 8); k::off_shell_fout(d, &a.v[0], &l[4]); }
    { let (l, d, _) = sp(&mut a.o, 6); k::off_shell_fout(d, &a.v[1], &l[4]); }
    { let (l, d, _) = sp(&mut a.o, 7); k::off_shell_fout(d, &a.v[3], &l[3]); }
    { let (l, d, _) = sp(&mut a.o, 18); k::off_shell_fout(d, &a.v[2], &l[4]); }
    { let (_, d, h) = sp(&mut a.s, 7); k::mul(d, &h[3], &h[1]); }
    { let (_, d, h) = sp(&mut a.s, 3); k::mul(d, &h[7], &h[9]); }
    { let (l, d, _) = sp(&mut a.s, 14); k::mul(d, &l[11], &l[12]); }
    { let (l, d, _) = sp(&mut a.s, 15); k::mul(d, &l[11], &l[5]); }
    { let (l, d, _) = sp(&mut a.s, 11); k::mul(d, &l[10], &l[9]); }
    { let (l, d, _) = sp(&mut a.s, 16); k::mul(d, &l[10], &l[13]); }
    { let (l, d, _) = sp(&mut a.s, 17); k::mul(d, &l[10], &l[12]); }
    { let (l, d, _) = sp(&mut a.s, 18); k::mul(d, &l[10], &l[5]); }
    { let (l, d, _) = sp(&mut a.s, 10); k::mul(d, &l[8], &l[9]); }
    { let (l, d, _) = sp(&mut a.s, 19); k::mul(d, &l[8], &l[13]); }
    { let (l, d, _) = sp(&mut a.s, 20); k::mul(d, &l[8], &l[12]); }
    { let (l, d, _) = sp(&mut a.s, 21); k::mul(d, &l[8], &l[5]); }
    { let (l, d, h) = sp(&mut a.s, 8); k::mul(d, &l[6], &h[0]); }
    { let (l, d, h) = sp(&mut a.s, 9); k::mul(d, &l[6], &h[3]); }
    { let (l, d, _) = sp(&mut a.s, 13); k::mul(d, &l[6], &l[12]); }
    { let (l, d, _) = sp(&mut a.s, 12); k::mul(d, &l[6], &l[5]); }
    { let (_, d, h) = sp(&mut a.i, 5); k::mul(d, &h[22], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 28); k::mul(d, &l[27], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 27); k::mul(d, &l[26], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 26); k::mul(d, &l[25], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 25); k::mul(d, &l[24], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 24); k::mul(d, &l[23], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 23); k::mul(d, &l[20], &cr[3]); }
    { let (_, d, h) = sp(&mut a.i, 20); k::mul(d, &h[1], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 22); k::mul(d, &l[17], &cr[3]); }
    { let (_, d, h) = sp(&mut a.i, 17); k::mul(d, &h[3], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 21); k::mul(d, &l[19], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 19); k::mul(d, &l[18], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 18); k::mul(d, &l[16], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 16); k::mul(d, &l[15], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 15); k::mul(d, &l[2], &cr[3]); }
    { let (_, d, h) = sp(&mut a.i, 2); k::mul(d, &h[7], &cr[3]); }
    { let (_, d, h) = sp(&mut a.i, 10); k::mul(d, &h[3], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 14); k::mul(d, &l[13], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 13); k::mul(d, &l[6], &cr[3]); }
    { let (_, d, h) = sp(&mut a.i, 6); k::mul(d, &h[5], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 12); k::mul(d, &l[11], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 11); k::mul(d, &l[9], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 9); k::mul(d, &l[8], &cr[3]); }
    { let (l, d, _) = sp(&mut a.i, 8); k::mul(d, &l[7], &cr[3]); }
    { let (l, d, _) = sp(&mut a.o, 17); k::mul(d, &l[15], &cr[3]); }
    { let (_, d, h) = sp(&mut a.o, 15); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.o, 16); k::mul(d, &l[14], &cr[3]); }
    { let (l, d, _) = sp(&mut a.o, 14); k::mul(d, &l[12], &cr[3]); }
    { let (_, d, h) = sp(&mut a.o, 12); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.o, 13); k::mul(d, &l[11], &cr[3]); }
    { let (l, d, _) = sp(&mut a.o, 11); k::mul(d, &l[9], &cr[3]); }
    { let (_, d, h) = sp(&mut a.o, 9); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.o, 10); k::mul(d, &l[8], &cr[3]); }
    { let (l, d, _) = sp(&mut a.o, 8); k::mul(d, &l[6], &cr[3]); }
    { let (_, d, h) = sp(&mut a.o, 6); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.o, 7); k::mul(d, &h[10], &cr[3]); }
    k::metric(&mut a.s[6], &a.v[5], &a.v[7]);
    k::metric(&mut a.s[5], &a.v[60], &a.v[8]);
    k::metric(&mut a.s[22], &a.v[17], &a.v[7]);
    k::metric(&mut a.s[23], &a.v[61], &a.v[8]);
    k::metric(&mut a.s[24], &a.v[18], &a.v[7]);
    k::metric(&mut a.s[25], &a.v[62], &a.v[8]);
    k::metric(&mut a.s[26], &a.v[19], &a.v[7]);
    k::metric(&mut a.s[27], &a.v[63], &a.v[8]);
    k::metric(&mut a.s[28], &a.v[20], &a.v[15]);
    k::metric(&mut a.s[29], &a.v[64], &a.v[9]);
    k::metric(&mut a.s[30], &a.v[21], &a.v[6]);
    k::metric(&mut a.s[31], &a.v[65], &a.v[16]);
    k::metric(&mut a.s[32], &a.v[22], &a.v[7]);
    k::metric(&mut a.s[33], &a.v[66], &a.v[8]);
    k::metric(&mut a.s[34], &a.v[23], &a.v[7]);
    k::metric(&mut a.s[35], &a.v[67], &a.v[8]);
    k::metric(&mut a.s[36], &a.v[24], &a.v[7]);
    k::metric(&mut a.s[37], &a.v[68], &a.v[8]);
    k::metric(&mut a.s[38], &a.v[25], &a.v[7]);
    k::metric(&mut a.s[39], &a.v[69], &a.v[8]);
    k::metric(&mut a.s[40], &a.v[26], &a.v[15]);
    k::metric(&mut a.s[41], &a.v[70], &a.v[9]);
    k::metric(&mut a.s[42], &a.v[21], &a.v[0]);
    k::metric(&mut a.s[43], &a.v[65], &a.v[10]);
    k::metric(&mut a.s[44], &a.v[27], &a.v[7]);
    k::metric(&mut a.s[45], &a.v[71], &a.v[8]);
    k::metric(&mut a.s[46], &a.v[28], &a.v[7]);
    k::metric(&mut a.s[47], &a.v[72], &a.v[8]);
    k::metric(&mut a.s[48], &a.v[29], &a.v[7]);
    k::metric(&mut a.s[49], &a.v[73], &a.v[8]);
    k::metric(&mut a.s[50], &a.v[30], &a.v[7]);
    k::metric(&mut a.s[51], &a.v[74], &a.v[8]);
    k::metric(&mut a.s[52], &a.v[31], &a.v[15]);
    k::metric(&mut a.s[53], &a.v[75], &a.v[9]);
    k::metric(&mut a.s[54], &a.v[21], &a.v[1]);
    k::metric(&mut a.s[55], &a.v[65], &a.v[11]);
    k::metric(&mut a.s[56], &a.v[32], &a.v[7]);
    k::metric(&mut a.s[57], &a.v[76], &a.v[8]);
    k::metric(&mut a.s[58], &a.v[33], &a.v[7]);
    k::metric(&mut a.s[59], &a.v[77], &a.v[8]);
    k::metric(&mut a.s[60], &a.v[34], &a.v[7]);
    k::metric(&mut a.s[61], &a.v[78], &a.v[8]);
    k::metric(&mut a.s[62], &a.v[35], &a.v[7]);
    k::metric(&mut a.s[63], &a.v[79], &a.v[8]);
    k::metric(&mut a.s[64], &a.v[36], &a.v[15]);
    k::metric(&mut a.s[65], &a.v[80], &a.v[9]);
    k::metric(&mut a.s[66], &a.v[21], &a.v[2]);
    k::metric(&mut a.s[67], &a.v[65], &a.v[12]);
    k::metric(&mut a.s[68], &a.v[37], &a.v[7]);
    k::metric(&mut a.s[69], &a.v[81], &a.v[8]);
    k::metric(&mut a.s[70], &a.v[38], &a.v[7]);
    k::metric(&mut a.s[71], &a.v[82], &a.v[8]);
    k::metric(&mut a.s[72], &a.v[39], &a.v[7]);
    k::metric(&mut a.s[73], &a.v[83], &a.v[8]);
    k::metric(&mut a.s[74], &a.v[40], &a.v[7]);
    k::metric(&mut a.s[75], &a.v[84], &a.v[8]);
    k::metric(&mut a.s[76], &a.v[20], &a.v[3]);
    k::metric(&mut a.s[77], &a.v[64], &a.v[13]);
    k::metric(&mut a.s[78], &a.v[41], &a.v[6]);
    k::metric(&mut a.s[79], &a.v[85], &a.v[16]);
    k::metric(&mut a.s[80], &a.v[42], &a.v[7]);
    k::metric(&mut a.s[81], &a.v[86], &a.v[8]);
    k::metric(&mut a.s[82], &a.v[43], &a.v[7]);
    k::metric(&mut a.s[83], &a.v[87], &a.v[8]);
    k::metric(&mut a.s[84], &a.v[44], &a.v[7]);
    k::metric(&mut a.s[85], &a.v[88], &a.v[8]);
    k::metric(&mut a.s[86], &a.v[45], &a.v[7]);
    k::metric(&mut a.s[87], &a.v[89], &a.v[8]);
    k::metric(&mut a.s[88], &a.v[26], &a.v[3]);
    k::metric(&mut a.s[89], &a.v[70], &a.v[13]);
    k::metric(&mut a.s[90], &a.v[41], &a.v[0]);
    k::metric(&mut a.s[91], &a.v[85], &a.v[10]);
    k::metric(&mut a.s[92], &a.v[46], &a.v[7]);
    k::metric(&mut a.s[93], &a.v[90], &a.v[8]);
    k::metric(&mut a.s[94], &a.v[47], &a.v[7]);
    k::metric(&mut a.s[95], &a.v[91], &a.v[8]);
    k::metric(&mut a.s[96], &a.v[48], &a.v[7]);
    k::metric(&mut a.s[97], &a.v[92], &a.v[8]);
    k::metric(&mut a.s[98], &a.v[49], &a.v[7]);
    k::metric(&mut a.s[99], &a.v[93], &a.v[8]);
    k::metric(&mut a.s[100], &a.v[31], &a.v[3]);
    k::metric(&mut a.s[101], &a.v[75], &a.v[13]);
    k::metric(&mut a.s[102], &a.v[41], &a.v[1]);
    k::metric(&mut a.s[103], &a.v[85], &a.v[11]);
    k::metric(&mut a.s[104], &a.v[50], &a.v[7]);
    k::metric(&mut a.s[105], &a.v[94], &a.v[8]);
    k::metric(&mut a.s[106], &a.v[51], &a.v[7]);
    k::metric(&mut a.s[107], &a.v[95], &a.v[8]);
    k::metric(&mut a.s[108], &a.v[52], &a.v[7]);
    k::metric(&mut a.s[109], &a.v[96], &a.v[8]);
    k::metric(&mut a.s[110], &a.v[53], &a.v[7]);
    k::metric(&mut a.s[111], &a.v[97], &a.v[8]);
    k::metric(&mut a.s[112], &a.v[36], &a.v[3]);
    k::metric(&mut a.s[113], &a.v[80], &a.v[13]);
    k::metric(&mut a.s[114], &a.v[41], &a.v[2]);
    k::metric(&mut a.s[115], &a.v[85], &a.v[12]);
    k::metric(&mut a.s[116], &a.v[5], &a.v[4]);
    k::metric(&mut a.s[117], &a.v[60], &a.v[14]);
    k::metric(&mut a.s[118], &a.v[17], &a.v[4]);
    k::metric(&mut a.s[119], &a.v[61], &a.v[14]);
    k::metric(&mut a.s[120], &a.v[18], &a.v[4]);
    k::metric(&mut a.s[121], &a.v[62], &a.v[14]);
    k::metric(&mut a.s[122], &a.v[19], &a.v[4]);
    k::metric(&mut a.s[123], &a.v[63], &a.v[14]);
    k::metric(&mut a.s[124], &a.v[54], &a.v[15]);
    k::metric(&mut a.s[125], &a.v[98], &a.v[9]);
    k::metric(&mut a.s[126], &a.v[55], &a.v[6]);
    k::metric(&mut a.s[127], &a.v[99], &a.v[16]);
    k::metric(&mut a.s[128], &a.v[22], &a.v[4]);
    k::metric(&mut a.s[129], &a.v[66], &a.v[14]);
    k::metric(&mut a.s[130], &a.v[23], &a.v[4]);
    k::metric(&mut a.s[131], &a.v[67], &a.v[14]);
    k::metric(&mut a.s[132], &a.v[24], &a.v[4]);
    k::metric(&mut a.s[133], &a.v[68], &a.v[14]);
    k::metric(&mut a.s[134], &a.v[25], &a.v[4]);
    k::metric(&mut a.s[135], &a.v[69], &a.v[14]);
    k::metric(&mut a.s[136], &a.v[56], &a.v[15]);
    k::metric(&mut a.s[137], &a.v[100], &a.v[9]);
    k::metric(&mut a.s[138], &a.v[55], &a.v[0]);
    k::metric(&mut a.s[139], &a.v[99], &a.v[10]);
    k::metric(&mut a.s[140], &a.v[27], &a.v[4]);
    k::metric(&mut a.s[141], &a.v[71], &a.v[14]);
    k::metric(&mut a.s[142], &a.v[28], &a.v[4]);
    k::metric(&mut a.s[143], &a.v[72], &a.v[14]);
    k::metric(&mut a.s[144], &a.v[29], &a.v[4]);
    k::metric(&mut a.s[145], &a.v[73], &a.v[14]);
    k::metric(&mut a.s[146], &a.v[30], &a.v[4]);
    k::metric(&mut a.s[147], &a.v[74], &a.v[14]);
    k::metric(&mut a.s[148], &a.v[57], &a.v[15]);
    k::metric(&mut a.s[149], &a.v[101], &a.v[9]);
    k::metric(&mut a.s[150], &a.v[55], &a.v[1]);
    k::metric(&mut a.s[151], &a.v[99], &a.v[11]);
    k::metric(&mut a.s[152], &a.v[32], &a.v[4]);
    k::metric(&mut a.s[153], &a.v[76], &a.v[14]);
    k::metric(&mut a.s[154], &a.v[33], &a.v[4]);
    k::metric(&mut a.s[155], &a.v[77], &a.v[14]);
    k::metric(&mut a.s[156], &a.v[34], &a.v[4]);
    k::metric(&mut a.s[157], &a.v[78], &a.v[14]);
    k::metric(&mut a.s[158], &a.v[35], &a.v[4]);
    k::metric(&mut a.s[159], &a.v[79], &a.v[14]);
    k::metric(&mut a.s[160], &a.v[58], &a.v[15]);
    k::metric(&mut a.s[161], &a.v[102], &a.v[9]);
    k::metric(&mut a.s[162], &a.v[55], &a.v[2]);
    k::metric(&mut a.s[163], &a.v[99], &a.v[12]);
    k::metric(&mut a.s[164], &a.v[37], &a.v[4]);
    k::metric(&mut a.s[165], &a.v[81], &a.v[14]);
    k::metric(&mut a.s[166], &a.v[38], &a.v[4]);
    k::metric(&mut a.s[167], &a.v[82], &a.v[14]);
    k::metric(&mut a.s[168], &a.v[39], &a.v[4]);
    k::metric(&mut a.s[169], &a.v[83], &a.v[14]);
    k::metric(&mut a.s[170], &a.v[40], &a.v[4]);
    k::metric(&mut a.s[171], &a.v[84], &a.v[14]);
    k::metric(&mut a.s[172], &a.v[54], &a.v[3]);
    k::metric(&mut a.s[173], &a.v[98], &a.v[13]);
    k::metric(&mut a.s[174], &a.v[59], &a.v[6]);
    k::metric(&mut a.s[175], &a.v[103], &a.v[16]);
    k::metric(&mut a.s[176], &a.v[42], &a.v[4]);
    k::metric(&mut a.s[177], &a.v[86], &a.v[14]);
    k::metric(&mut a.s[178], &a.v[43], &a.v[4]);
    k::metric(&mut a.s[179], &a.v[87], &a.v[14]);
    k::metric(&mut a.s[180], &a.v[44], &a.v[4]);
    k::metric(&mut a.s[181], &a.v[88], &a.v[14]);
    k::metric(&mut a.s[182], &a.v[45], &a.v[4]);
    k::metric(&mut a.s[183], &a.v[89], &a.v[14]);
    k::metric(&mut a.s[184], &a.v[56], &a.v[3]);
    k::metric(&mut a.s[185], &a.v[100], &a.v[13]);
    k::metric(&mut a.s[186], &a.v[59], &a.v[0]);
    k::metric(&mut a.s[187], &a.v[103], &a.v[10]);
    k::metric(&mut a.s[188], &a.v[46], &a.v[4]);
    k::metric(&mut a.s[189], &a.v[90], &a.v[14]);
    k::metric(&mut a.s[190], &a.v[47], &a.v[4]);
    k::metric(&mut a.s[191], &a.v[91], &a.v[14]);
    k::metric(&mut a.s[192], &a.v[48], &a.v[4]);
    k::metric(&mut a.s[193], &a.v[92], &a.v[14]);
    k::metric(&mut a.s[194], &a.v[49], &a.v[4]);
    k::metric(&mut a.s[195], &a.v[93], &a.v[14]);
    k::metric(&mut a.s[196], &a.v[57], &a.v[3]);
    k::metric(&mut a.s[197], &a.v[101], &a.v[13]);
    k::metric(&mut a.s[198], &a.v[59], &a.v[1]);
    k::metric(&mut a.s[199], &a.v[103], &a.v[11]);
    k::metric(&mut a.s[200], &a.v[50], &a.v[4]);
    k::metric(&mut a.s[201], &a.v[94], &a.v[14]);
    k::metric(&mut a.s[202], &a.v[51], &a.v[4]);
    k::metric(&mut a.s[203], &a.v[95], &a.v[14]);
    k::metric(&mut a.s[204], &a.v[52], &a.v[4]);
    k::metric(&mut a.s[205], &a.v[96], &a.v[14]);
    k::metric(&mut a.s[206], &a.v[53], &a.v[4]);
    k::metric(&mut a.s[207], &a.v[97], &a.v[14]);
    k::metric(&mut a.s[208], &a.v[58], &a.v[3]);
    k::metric(&mut a.s[209], &a.v[102], &a.v[13]);
    k::metric(&mut a.s[210], &a.v[59], &a.v[2]);
    k::metric(&mut a.s[211], &a.v[103], &a.v[12]);
    { let (l, d, _) = sp(&mut a.s, 212); k::mul(d, &l[6], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 6); k::mul(d, &l[5], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 213); k::mul(d, &l[22], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 22); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 214); k::mul(d, &l[24], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 24); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 215); k::mul(d, &l[26], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 26); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 216); k::mul(d, &l[7], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 7); k::mul(d, &h[20], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 28); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 217); k::mul(d, &l[30], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 30); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 218); k::mul(d, &l[32], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 32); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 219); k::mul(d, &l[34], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 34); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 220); k::mul(d, &l[36], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 36); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 221); k::mul(d, &l[38], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 38); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 222); k::mul(d, &l[3], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 3); k::mul(d, &h[36], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 40); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 223); k::mul(d, &l[42], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 42); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 224); k::mul(d, &l[44], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 44); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 225); k::mul(d, &l[46], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 46); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 226); k::mul(d, &l[48], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 48); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 227); k::mul(d, &l[50], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 50); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 228); k::mul(d, &l[14], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 14); k::mul(d, &h[37], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 52); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 229); k::mul(d, &l[54], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 54); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 230); k::mul(d, &l[56], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 56); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 231); k::mul(d, &l[58], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 58); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 232); k::mul(d, &l[60], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 60); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 233); k::mul(d, &l[62], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 62); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 234); k::mul(d, &l[15], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 15); k::mul(d, &h[48], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 64); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 235); k::mul(d, &l[66], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 66); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 236); k::mul(d, &l[68], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 68); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 237); k::mul(d, &l[70], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 70); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 238); k::mul(d, &l[72], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 72); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 239); k::mul(d, &l[74], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 74); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 240); k::mul(d, &l[11], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 11); k::mul(d, &h[64], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 76); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 241); k::mul(d, &l[78], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 78); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 242); k::mul(d, &l[80], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 80); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 243); k::mul(d, &l[82], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 82); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 244); k::mul(d, &l[84], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 84); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 245); k::mul(d, &l[86], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 86); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 246); k::mul(d, &l[16], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 16); k::mul(d, &h[71], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 88); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 247); k::mul(d, &l[90], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 90); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 248); k::mul(d, &l[92], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 92); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 249); k::mul(d, &l[94], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 94); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 250); k::mul(d, &l[96], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 96); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 251); k::mul(d, &l[98], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 98); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 252); k::mul(d, &l[17], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 17); k::mul(d, &h[82], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 100); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 253); k::mul(d, &l[102], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 102); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 254); k::mul(d, &l[104], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 104); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 255); k::mul(d, &l[106], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 106); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 256); k::mul(d, &l[108], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 108); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 257); k::mul(d, &l[110], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 110); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 258); k::mul(d, &l[18], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 18); k::mul(d, &h[93], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 112); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 259); k::mul(d, &l[114], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 114); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 260); k::mul(d, &l[116], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 116); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 261); k::mul(d, &l[118], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 118); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 262); k::mul(d, &l[120], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 120); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 263); k::mul(d, &l[122], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 122); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 264); k::mul(d, &l[10], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 10); k::mul(d, &h[113], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 124); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 265); k::mul(d, &l[126], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 126); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 266); k::mul(d, &l[128], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 128); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 267); k::mul(d, &l[130], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 130); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 268); k::mul(d, &l[132], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 132); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 269); k::mul(d, &l[134], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 134); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 270); k::mul(d, &l[19], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 19); k::mul(d, &h[116], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 136); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 271); k::mul(d, &l[138], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 138); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 272); k::mul(d, &l[140], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 140); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 273); k::mul(d, &l[142], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 142); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 274); k::mul(d, &l[144], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 144); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 275); k::mul(d, &l[146], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 146); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 276); k::mul(d, &l[20], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 20); k::mul(d, &h[127], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 148); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 277); k::mul(d, &l[150], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 150); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 278); k::mul(d, &l[152], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 152); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 279); k::mul(d, &l[154], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 154); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 280); k::mul(d, &l[156], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 156); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 281); k::mul(d, &l[158], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 158); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 282); k::mul(d, &l[21], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 21); k::mul(d, &h[138], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 160); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 283); k::mul(d, &l[162], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 162); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 284); k::mul(d, &l[164], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 164); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 285); k::mul(d, &l[166], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 166); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 286); k::mul(d, &l[168], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 168); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 287); k::mul(d, &l[170], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 170); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 288); k::mul(d, &l[8], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 8); k::mul(d, &h[163], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 172); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 289); k::mul(d, &l[174], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 174); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 290); k::mul(d, &l[176], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 176); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 291); k::mul(d, &l[178], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 178); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 292); k::mul(d, &l[180], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 180); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 293); k::mul(d, &l[182], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 182); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 294); k::mul(d, &l[9], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 9); k::mul(d, &h[174], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 184); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 295); k::mul(d, &l[186], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 186); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 296); k::mul(d, &l[188], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 188); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 297); k::mul(d, &l[190], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 190); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 298); k::mul(d, &l[192], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 192); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 299); k::mul(d, &l[194], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 194); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 300); k::mul(d, &l[13], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 13); k::mul(d, &h[182], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 196); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 301); k::mul(d, &l[198], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 198); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 302); k::mul(d, &l[200], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 200); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 303); k::mul(d, &l[202], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 202); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 304); k::mul(d, &l[204], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 204); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 305); k::mul(d, &l[206], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 206); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 306); k::mul(d, &l[12], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 12); k::mul(d, &h[195], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 208); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 307); k::mul(d, &l[210], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 210); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.i, 7); k::mul(d, &l[5], &cc[0]); }
    { let (_, d, h) = sp(&mut a.i, 5); k::mul(d, &h[22], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 28); k::mul(d, &l[27], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 27); k::mul(d, &l[26], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 26); k::mul(d, &l[25], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 25); k::mul(d, &l[24], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 24); k::mul(d, &l[23], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 23); k::mul(d, &l[20], &cc[0]); }
    { let (_, d, h) = sp(&mut a.i, 20); k::mul(d, &h[1], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 22); k::mul(d, &l[17], &cc[0]); }
    { let (_, d, h) = sp(&mut a.i, 17); k::mul(d, &h[3], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 21); k::mul(d, &l[19], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 19); k::mul(d, &l[18], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 18); k::mul(d, &l[16], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 16); k::mul(d, &l[15], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 15); k::mul(d, &l[2], &cc[0]); }
    { let (_, d, h) = sp(&mut a.i, 2); k::mul(d, &h[7], &cc[0]); }
    { let (_, d, h) = sp(&mut a.i, 10); k::mul(d, &h[3], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 14); k::mul(d, &l[13], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 13); k::mul(d, &l[6], &cc[0]); }
    { let (_, d, h) = sp(&mut a.i, 6); k::mul(d, &h[5], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 12); k::mul(d, &l[11], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 11); k::mul(d, &l[9], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 9); k::mul(d, &l[8], &cc[0]); }
    { let (l, d, _) = sp(&mut a.o, 18); k::mul(d, &l[17], &cc[0]); }
    { let (l, d, _) = sp(&mut a.o, 17); k::mul(d, &l[15], &cc[0]); }
    { let (_, d, h) = sp(&mut a.o, 15); k::mul(d, &h[0], &cc[0]); }
    { let (l, d, _) = sp(&mut a.o, 16); k::mul(d, &l[14], &cc[0]); }
    { let (l, d, _) = sp(&mut a.o, 14); k::mul(d, &l[12], &cc[0]); }
    { let (_, d, h) = sp(&mut a.o, 12); k::mul(d, &h[0], &cc[0]); }
    { let (l, d, _) = sp(&mut a.o, 13); k::mul(d, &l[11], &cc[0]); }
    { let (l, d, _) = sp(&mut a.o, 11); k::mul(d, &l[9], &cc[0]); }
    { let (_, d, h) = sp(&mut a.o, 9); k::mul(d, &h[0], &cc[0]); }
    { let (l, d, _) = sp(&mut a.o, 10); k::mul(d, &l[8], &cc[0]); }
    { let (l, d, _) = sp(&mut a.o, 8); k::mul(d, &l[6], &cc[0]); }
    { let (_, d, h) = sp(&mut a.o, 6); k::mul(d, &h[0], &cc[0]); }
    { let (l, d, _) = sp(&mut a.i, 8); k::propagate_fin(d, &l[7], &mm[5], &cr[6], &cr[7]); }
    { let (l, d, _) = sp(&mut a.i, 7); k::propagate_fin(d, &l[5], &mm[14], &cr[2], &cr[14]); }
    { let (_, d, h) = sp(&mut a.i, 5); k::propagate_fin(d, &h[22], &mm[17], &cr[8], &cr[18]); }
    { let (l, d, _) = sp(&mut a.i, 28); k::propagate_fin(d, &l[27], &mm[19], &cr[8], &cr[18]); }
    { let (l, d, _) = sp(&mut a.i, 27); k::propagate_fin(d, &l[26], &mm[5], &cr[6], &cr[7]); }
    { let (l, d, _) = sp(&mut a.i, 26); k::propagate_fin(d, &l[25], &mm[14], &cr[2], &cr[14]); }
    { let (l, d, _) = sp(&mut a.i, 25); k::propagate_fin(d, &l[24], &mm[17], &cr[8], &cr[18]); }
    { let (l, d, _) = sp(&mut a.i, 24); k::propagate_fin(d, &l[23], &mm[5], &cr[6], &cr[7]); }
    { let (l, d, _) = sp(&mut a.i, 23); k::propagate_fin(d, &l[20], &mm[17], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 20); k::propagate_fin(d, &h[1], &mm[5], &cr[6], &cr[7]); }
    { let (l, d, _) = sp(&mut a.i, 22); k::propagate_fin(d, &l[17], &mm[17], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 17); k::propagate_fin(d, &h[3], &mm[5], &cr[6], &cr[7]); }
    { let (l, d, _) = sp(&mut a.i, 21); k::propagate_fin(d, &l[19], &mm[14], &cr[2], &cr[14]); }
    { let (l, d, _) = sp(&mut a.i, 19); k::propagate_fin(d, &l[18], &mm[19], &cr[8], &cr[18]); }
    { let (l, d, _) = sp(&mut a.i, 18); k::propagate_fin(d, &l[16], &mm[5], &cr[6], &cr[7]); }
    { let (l, d, _) = sp(&mut a.i, 16); k::propagate_fin(d, &l[15], &mm[14], &cr[2], &cr[14]); }
    { let (l, d, _) = sp(&mut a.i, 15); k::propagate_fin(d, &l[2], &mm[5], &cr[6], &cr[7]); }
    { let (_, d, h) = sp(&mut a.i, 2); k::propagate_fin(d, &h[7], &mm[5], &cr[6], &cr[7]); }
    { let (_, d, h) = sp(&mut a.i, 10); k::propagate_fin(d, &h[3], &mm[17], &cr[8], &cr[18]); }
    { let (l, d, _) = sp(&mut a.i, 14); k::propagate_fin(d, &l[13], &mm[19], &cr[8], &cr[18]); }
    { let (l, d, _) = sp(&mut a.i, 13); k::propagate_fin(d, &l[6], &mm[17], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.i, 6); k::propagate_fin(d, &h[5], &mm[17], &cr[8], &cr[18]); }
    { let (l, d, _) = sp(&mut a.i, 12); k::propagate_fin(d, &l[11], &mm[17], &cr[8], &cr[18]); }
    { let (l, d, _) = sp(&mut a.i, 11); k::propagate_fin(d, &l[9], &mm[19], &cr[8], &cr[18]); }
    { let (_, d, h) = sp(&mut a.o, 7); k::propagate_fout(d, &h[10], &mm[12], &cr[0], &cr[13]); }
    { let (l, d, _) = sp(&mut a.o, 18); k::propagate_fout(d, &l[17], &mm[15], &cr[1], &cr[15]); }
    { let (l, d, _) = sp(&mut a.o, 17); k::propagate_fout(d, &l[15], &mm[12], &cr[0], &cr[13]); }
    { let (_, d, h) = sp(&mut a.o, 15); k::propagate_fout(d, &h[0], &mm[12], &cr[0], &cr[13]); }
    { let (l, d, _) = sp(&mut a.o, 16); k::propagate_fout(d, &l[14], &mm[15], &cr[1], &cr[15]); }
    { let (l, d, _) = sp(&mut a.o, 14); k::propagate_fout(d, &l[12], &mm[12], &cr[0], &cr[13]); }
    { let (_, d, h) = sp(&mut a.o, 12); k::propagate_fout(d, &h[0], &mm[12], &cr[0], &cr[13]); }
    { let (l, d, _) = sp(&mut a.o, 13); k::propagate_fout(d, &l[11], &mm[15], &cr[1], &cr[15]); }
    { let (l, d, _) = sp(&mut a.o, 11); k::propagate_fout(d, &l[9], &mm[12], &cr[0], &cr[13]); }
    { let (_, d, h) = sp(&mut a.o, 9); k::propagate_fout(d, &h[0], &mm[12], &cr[0], &cr[13]); }
    { let (l, d, _) = sp(&mut a.o, 10); k::propagate_fout(d, &l[8], &mm[15], &cr[1], &cr[15]); }
    { let (l, d, _) = sp(&mut a.o, 8); k::propagate_fout(d, &l[6], &mm[12], &cr[0], &cr[13]); }
    { let (l, d, _) = sp(&mut a.s, 308); k::mul(d, &cc[0], &l[212]); }
    { let (_, d, h) = sp(&mut a.s, 212); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 213); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 214); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 215); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 216); k::mul(d, &cc[0], &l[7]); }
    { let (_, d, h) = sp(&mut a.s, 7); k::mul(d, &cc[0], &h[209]); }
    { let (_, d, h) = sp(&mut a.s, 217); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 218); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 219); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 220); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 221); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 222); k::mul(d, &cc[0], &l[3]); }
    { let (_, d, h) = sp(&mut a.s, 3); k::mul(d, &cc[0], &h[219]); }
    { let (_, d, h) = sp(&mut a.s, 223); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 224); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 225); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 226); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 227); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 228); k::mul(d, &cc[0], &l[14]); }
    { let (_, d, h) = sp(&mut a.s, 14); k::mul(d, &cc[0], &h[214]); }
    { let (_, d, h) = sp(&mut a.s, 229); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 230); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 231); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 232); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 233); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 234); k::mul(d, &cc[0], &l[15]); }
    { let (_, d, h) = sp(&mut a.s, 15); k::mul(d, &cc[0], &h[219]); }
    { let (_, d, h) = sp(&mut a.s, 235); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 236); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 237); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 238); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 239); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 240); k::mul(d, &cc[0], &l[11]); }
    { let (_, d, h) = sp(&mut a.s, 11); k::mul(d, &cc[0], &h[229]); }
    { let (_, d, h) = sp(&mut a.s, 241); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 242); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 243); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 244); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 245); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 246); k::mul(d, &cc[0], &l[16]); }
    { let (_, d, h) = sp(&mut a.s, 16); k::mul(d, &cc[0], &h[230]); }
    { let (_, d, h) = sp(&mut a.s, 247); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 248); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 249); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 250); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 251); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 252); k::mul(d, &cc[0], &l[17]); }
    { let (_, d, h) = sp(&mut a.s, 17); k::mul(d, &cc[0], &h[235]); }
    { let (_, d, h) = sp(&mut a.s, 253); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 254); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 255); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 256); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 257); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 258); k::mul(d, &cc[0], &l[18]); }
    { let (_, d, h) = sp(&mut a.s, 18); k::mul(d, &cc[0], &h[240]); }
    { let (_, d, h) = sp(&mut a.s, 259); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 260); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 261); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 262); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 263); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 264); k::mul(d, &cc[0], &l[10]); }
    { let (_, d, h) = sp(&mut a.s, 10); k::mul(d, &cc[0], &h[254]); }
    { let (_, d, h) = sp(&mut a.s, 265); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 266); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 267); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 268); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 269); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 270); k::mul(d, &cc[0], &l[19]); }
    { let (_, d, h) = sp(&mut a.s, 19); k::mul(d, &cc[0], &h[251]); }
    { let (_, d, h) = sp(&mut a.s, 271); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 272); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 273); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 274); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 275); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 276); k::mul(d, &cc[0], &l[20]); }
    { let (_, d, h) = sp(&mut a.s, 20); k::mul(d, &cc[0], &h[256]); }
    { let (_, d, h) = sp(&mut a.s, 277); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 278); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 279); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 280); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 281); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 282); k::mul(d, &cc[0], &l[21]); }
    { let (_, d, h) = sp(&mut a.s, 21); k::mul(d, &cc[0], &h[261]); }
    { let (_, d, h) = sp(&mut a.s, 283); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 284); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 285); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 286); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 287); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 288); k::mul(d, &cc[0], &l[8]); }
    { let (_, d, h) = sp(&mut a.s, 8); k::mul(d, &cc[0], &h[280]); }
    { let (_, d, h) = sp(&mut a.s, 289); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 290); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 291); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 292); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 293); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 294); k::mul(d, &cc[0], &l[9]); }
    { let (_, d, h) = sp(&mut a.s, 9); k::mul(d, &cc[0], &h[285]); }
    { let (_, d, h) = sp(&mut a.s, 295); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 296); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 297); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 298); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 299); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 300); k::mul(d, &cc[0], &l[13]); }
    { let (_, d, h) = sp(&mut a.s, 13); k::mul(d, &cc[0], &h[287]); }
    { let (_, d, h) = sp(&mut a.s, 301); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 302); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 303); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 304); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 305); k::mul(d, &cc[4], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 306); k::mul(d, &cc[0], &l[12]); }
    { let (_, d, h) = sp(&mut a.s, 12); k::mul(d, &cc[0], &h[294]); }
    { let (_, d, h) = sp(&mut a.s, 307); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 4); k::mul(d, &h[207], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 309); k::mul(d, &l[213], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 310); k::mul(d, &l[214], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 311); k::mul(d, &l[215], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 312); k::mul(d, &l[216], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 313); k::mul(d, &l[7], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 314); k::mul(d, &l[217], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 315); k::mul(d, &l[218], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 316); k::mul(d, &l[219], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 317); k::mul(d, &l[220], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 318); k::mul(d, &l[221], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 319); k::mul(d, &l[222], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 320); k::mul(d, &l[3], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 321); k::mul(d, &l[223], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 322); k::mul(d, &l[224], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 323); k::mul(d, &l[225], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 324); k::mul(d, &l[226], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 325); k::mul(d, &l[227], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 326); k::mul(d, &l[228], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 327); k::mul(d, &l[14], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 328); k::mul(d, &l[229], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 329); k::mul(d, &l[230], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 330); k::mul(d, &l[231], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 331); k::mul(d, &l[232], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 332); k::mul(d, &l[233], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 333); k::mul(d, &l[234], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 334); k::mul(d, &l[15], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 335); k::mul(d, &l[235], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 336); k::mul(d, &l[236], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 337); k::mul(d, &l[237], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 338); k::mul(d, &l[238], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 339); k::mul(d, &l[239], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 340); k::mul(d, &l[240], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 341); k::mul(d, &l[11], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 342); k::mul(d, &l[241], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 343); k::mul(d, &l[242], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 344); k::mul(d, &l[243], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 345); k::mul(d, &l[244], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 346); k::mul(d, &l[245], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 347); k::mul(d, &l[246], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 348); k::mul(d, &l[16], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 349); k::mul(d, &l[247], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 350); k::mul(d, &l[248], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 351); k::mul(d, &l[249], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 352); k::mul(d, &l[250], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 353); k::mul(d, &l[251], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 354); k::mul(d, &l[252], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 355); k::mul(d, &l[17], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 356); k::mul(d, &l[253], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 357); k::mul(d, &l[254], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 358); k::mul(d, &l[255], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 359); k::mul(d, &l[256], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 360); k::mul(d, &l[257], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 361); k::mul(d, &l[258], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 362); k::mul(d, &l[18], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 363); k::mul(d, &l[259], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 364); k::mul(d, &l[260], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 365); k::mul(d, &l[261], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 366); k::mul(d, &l[262], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 367); k::mul(d, &l[263], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 368); k::mul(d, &l[264], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 369); k::mul(d, &l[10], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 370); k::mul(d, &l[265], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 371); k::mul(d, &l[266], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 372); k::mul(d, &l[267], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 373); k::mul(d, &l[268], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 374); k::mul(d, &l[269], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 375); k::mul(d, &l[270], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 376); k::mul(d, &l[19], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 377); k::mul(d, &l[271], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 378); k::mul(d, &l[272], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 379); k::mul(d, &l[273], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 380); k::mul(d, &l[274], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 381); k::mul(d, &l[275], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 382); k::mul(d, &l[276], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 383); k::mul(d, &l[20], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 384); k::mul(d, &l[277], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 385); k::mul(d, &l[278], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 386); k::mul(d, &l[279], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 387); k::mul(d, &l[280], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 388); k::mul(d, &l[281], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 389); k::mul(d, &l[282], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 390); k::mul(d, &l[21], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 391); k::mul(d, &l[283], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 392); k::mul(d, &l[284], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 393); k::mul(d, &l[285], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 394); k::mul(d, &l[286], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 395); k::mul(d, &l[287], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 396); k::mul(d, &l[288], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 397); k::mul(d, &l[8], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 398); k::mul(d, &l[289], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 399); k::mul(d, &l[290], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 400); k::mul(d, &l[291], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 401); k::mul(d, &l[292], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 402); k::mul(d, &l[293], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 403); k::mul(d, &l[294], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 404); k::mul(d, &l[9], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 405); k::mul(d, &l[295], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 406); k::mul(d, &l[296], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 407); k::mul(d, &l[297], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 408); k::mul(d, &l[298], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 409); k::mul(d, &l[299], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 410); k::mul(d, &l[300], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 411); k::mul(d, &l[13], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 412); k::mul(d, &l[301], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 413); k::mul(d, &l[302], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 414); k::mul(d, &l[303], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 415); k::mul(d, &l[304], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 416); k::mul(d, &l[305], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 417); k::mul(d, &l[306], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 418); k::mul(d, &l[12], &cr[19]); }
    k::gamma_vout(&mut a.v[103], &a.o[1], &a.i[8], false);
    k::gamma_vout(&mut a.v[59], &a.o[7], &a.i[0], false);
    k::gamma_vout(&mut a.v[102], &a.o[0], &a.i[7], false);
    k::gamma_vout(&mut a.v[58], &a.o[18], &a.i[1], false);
    k::gamma_vout(&mut a.v[97], &a.o[2], &a.i[5], true);
    k::gamma_vout(&mut a.v[53], &a.o[2], &a.i[28], true);
    k::gamma_vout(&mut a.v[96], &a.o[1], &a.i[27], false);
    k::gamma_vout(&mut a.v[52], &a.o[17], &a.i[0], false);
    k::gamma_vout(&mut a.v[95], &a.o[0], &a.i[26], false);
    k::gamma_vout(&mut a.v[51], &a.o[18], &a.i[3], false);
    k::gamma_vout(&mut a.v[94], &a.o[2], &a.i[25], true);
    k::gamma_vout(&mut a.v[50], &a.o[1], &a.i[24], false);
    k::gamma_vout(&mut a.v[101], &a.o[15], &a.i[0], false);
    k::gamma_vout(&mut a.v[57], &a.o[3], &a.i[7], false);
    k::gamma_vout(&mut a.v[93], &a.o[16], &a.i[1], false);
    k::gamma_vout(&mut a.v[49], &a.o[2], &a.i[23], true);
    k::gamma_vout(&mut a.v[92], &a.o[1], &a.i[20], false);
    k::gamma_vout(&mut a.v[48], &a.o[14], &a.i[0], false);
    k::gamma_vout(&mut a.v[91], &a.o[3], &a.i[26], false);
    k::gamma_vout(&mut a.v[47], &a.o[16], &a.i[3], false);
    k::gamma_vout(&mut a.v[90], &a.o[2], &a.i[22], true);
    k::gamma_vout(&mut a.v[46], &a.o[4], &a.i[17], false);
    k::gamma_vout(&mut a.v[100], &a.o[12], &a.i[4], false);
    k::gamma_vout(&mut a.v[56], &a.o[0], &a.i[21], false);
    k::gamma_vout(&mut a.v[89], &a.o[13], &a.i[1], false);
    k::gamma_vout(&mut a.v[45], &a.o[2], &a.i[19], true);
    k::gamma_vout(&mut a.v[88], &a.o[4], &a.i[18], false);
    k::gamma_vout(&mut a.v[44], &a.o[11], &a.i[4], false);
    k::gamma_vout(&mut a.v[87], &a.o[0], &a.i[16], false);
    k::gamma_vout(&mut a.v[43], &a.o[13], &a.i[3], false);
    k::gamma_vout(&mut a.v[86], &a.o[4], &a.i[15], false);
    k::gamma_vout(&mut a.v[42], &a.o[9], &a.i[4], false);
    k::gamma_vout(&mut a.v[98], &a.o[3], &a.i[21], false);
    k::gamma_vout(&mut a.v[54], &a.o[10], &a.i[1], false);
    k::gamma_vout(&mut a.v[84], &a.o[4], &a.i[2], false);
    k::gamma_vout(&mut a.v[40], &a.o[8], &a.i[4], false);
    k::gamma_vout(&mut a.v[83], &a.o[3], &a.i[16], false);
    k::gamma_vout(&mut a.v[39], &a.o[10], &a.i[3], false);
    k::gamma_vout(&mut a.v[82], &a.o[5], &a.i[10], true);
    k::gamma_vout(&mut a.v[38], &a.o[5], &a.i[14], true);
    k::gamma_vout(&mut a.v[81], &a.o[5], &a.i[13], true);
    k::gamma_vout(&mut a.v[37], &a.o[5], &a.i[6], true);
    k::gamma_vout(&mut a.v[99], &a.o[5], &a.i[12], true);
    k::gamma_vout(&mut a.v[55], &a.o[5], &a.i[11], true);
    k::ffv_vout(&mut a.v[79], &a.o[1], &a.i[8], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[35], &a.o[7], &a.i[0], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[78], &a.o[0], &a.i[7], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[34], &a.o[18], &a.i[1], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[77], &a.o[2], &a.i[5], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[33], &a.o[2], &a.i[28], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[76], &a.o[1], &a.i[27], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[32], &a.o[17], &a.i[0], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[74], &a.o[0], &a.i[26], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[30], &a.o[18], &a.i[3], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[73], &a.o[2], &a.i[25], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[29], &a.o[1], &a.i[24], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[72], &a.o[15], &a.i[0], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[28], &a.o[3], &a.i[7], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[71], &a.o[16], &a.i[1], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[27], &a.o[2], &a.i[23], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[69], &a.o[1], &a.i[20], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[25], &a.o[14], &a.i[0], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[68], &a.o[3], &a.i[26], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[24], &a.o[16], &a.i[3], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[67], &a.o[2], &a.i[22], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[23], &a.o[4], &a.i[17], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[66], &a.o[12], &a.i[4], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[22], &a.o[0], &a.i[21], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[63], &a.o[13], &a.i[1], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[19], &a.o[2], &a.i[19], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[62], &a.o[4], &a.i[18], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[18], &a.o[11], &a.i[4], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[61], &a.o[0], &a.i[16], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[17], &a.o[13], &a.i[3], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[60], &a.o[4], &a.i[15], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[5], &a.o[9], &a.i[4], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[85], &a.o[3], &a.i[21], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[41], &a.o[10], &a.i[1], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[80], &a.o[4], &a.i[2], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[36], &a.o[8], &a.i[4], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[75], &a.o[3], &a.i[16], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[31], &a.o[10], &a.i[3], &cc[5], &cc[6], false);
    k::ffv_vout(&mut a.v[70], &a.o[5], &a.i[10], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[26], &a.o[5], &a.i[14], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[64], &a.o[5], &a.i[13], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[20], &a.o[5], &a.i[6], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[65], &a.o[5], &a.i[12], &cc[5], &cc[6], true);
    k::ffv_vout(&mut a.v[21], &a.o[5], &a.i[11], &cc[5], &cc[6], true);
    k::metric(&mut a.s[2], &a.v[103], &a.v[7]);
    k::metric(&mut a.s[1], &a.v[79], &a.v[8]);
    k::metric(&mut a.s[419], &a.v[59], &a.v[7]);
    k::metric(&mut a.s[420], &a.v[35], &a.v[8]);
    k::metric(&mut a.s[421], &a.v[102], &a.v[7]);
    k::metric(&mut a.s[422], &a.v[78], &a.v[8]);
    k::metric(&mut a.s[423], &a.v[58], &a.v[7]);
    k::metric(&mut a.s[424], &a.v[34], &a.v[8]);
    k::metric(&mut a.s[425], &a.v[97], &a.v[15]);
    k::metric(&mut a.s[426], &a.v[77], &a.v[9]);
    k::metric(&mut a.s[427], &a.v[53], &a.v[6]);
    k::metric(&mut a.s[428], &a.v[33], &a.v[16]);
    k::metric(&mut a.s[429], &a.v[96], &a.v[7]);
    k::metric(&mut a.s[430], &a.v[76], &a.v[8]);
    k::metric(&mut a.s[431], &a.v[52], &a.v[7]);
    k::metric(&mut a.s[432], &a.v[32], &a.v[8]);
    k::metric(&mut a.s[433], &a.v[95], &a.v[7]);
    k::metric(&mut a.s[434], &a.v[74], &a.v[8]);
    k::metric(&mut a.s[435], &a.v[51], &a.v[7]);
    k::metric(&mut a.s[436], &a.v[30], &a.v[8]);
    k::metric(&mut a.s[437], &a.v[94], &a.v[15]);
    k::metric(&mut a.s[438], &a.v[73], &a.v[9]);
    k::metric(&mut a.s[439], &a.v[53], &a.v[0]);
    k::metric(&mut a.s[440], &a.v[33], &a.v[10]);
    k::metric(&mut a.s[441], &a.v[50], &a.v[7]);
    k::metric(&mut a.s[442], &a.v[29], &a.v[8]);
    k::metric(&mut a.s[443], &a.v[101], &a.v[7]);
    k::metric(&mut a.s[444], &a.v[72], &a.v[8]);
    k::metric(&mut a.s[445], &a.v[57], &a.v[7]);
    k::metric(&mut a.s[446], &a.v[28], &a.v[8]);
    k::metric(&mut a.s[447], &a.v[93], &a.v[7]);
    k::metric(&mut a.s[448], &a.v[71], &a.v[8]);
    k::metric(&mut a.s[449], &a.v[49], &a.v[15]);
    k::metric(&mut a.s[450], &a.v[27], &a.v[9]);
    k::metric(&mut a.s[451], &a.v[53], &a.v[1]);
    k::metric(&mut a.s[452], &a.v[33], &a.v[11]);
    k::metric(&mut a.s[453], &a.v[92], &a.v[7]);
    k::metric(&mut a.s[454], &a.v[69], &a.v[8]);
    k::metric(&mut a.s[455], &a.v[48], &a.v[7]);
    k::metric(&mut a.s[456], &a.v[25], &a.v[8]);
    k::metric(&mut a.s[457], &a.v[91], &a.v[7]);
    k::metric(&mut a.s[458], &a.v[68], &a.v[8]);
    k::metric(&mut a.s[459], &a.v[47], &a.v[7]);
    k::metric(&mut a.s[460], &a.v[24], &a.v[8]);
    k::metric(&mut a.s[461], &a.v[90], &a.v[15]);
    k::metric(&mut a.s[462], &a.v[67], &a.v[9]);
    k::metric(&mut a.s[463], &a.v[53], &a.v[2]);
    k::metric(&mut a.s[464], &a.v[33], &a.v[12]);
    k::metric(&mut a.s[465], &a.v[46], &a.v[7]);
    k::metric(&mut a.s[466], &a.v[23], &a.v[8]);
    k::metric(&mut a.s[467], &a.v[100], &a.v[7]);
    k::metric(&mut a.s[468], &a.v[66], &a.v[8]);
    k::metric(&mut a.s[469], &a.v[56], &a.v[7]);
    k::metric(&mut a.s[470], &a.v[22], &a.v[8]);
    k::metric(&mut a.s[471], &a.v[89], &a.v[7]);
    k::metric(&mut a.s[472], &a.v[63], &a.v[8]);
    k::metric(&mut a.s[473], &a.v[97], &a.v[3]);
    k::metric(&mut a.s[474], &a.v[77], &a.v[13]);
    k::metric(&mut a.s[475], &a.v[45], &a.v[6]);
    k::metric(&mut a.s[476], &a.v[19], &a.v[16]);
    k::metric(&mut a.s[477], &a.v[88], &a.v[7]);
    k::metric(&mut a.s[478], &a.v[62], &a.v[8]);
    k::metric(&mut a.s[479], &a.v[44], &a.v[7]);
    k::metric(&mut a.s[480], &a.v[18], &a.v[8]);
    k::metric(&mut a.s[481], &a.v[87], &a.v[7]);
    k::metric(&mut a.s[482], &a.v[61], &a.v[8]);
    k::metric(&mut a.s[483], &a.v[43], &a.v[7]);
    k::metric(&mut a.s[484], &a.v[17], &a.v[8]);
    k::metric(&mut a.s[485], &a.v[94], &a.v[3]);
    k::metric(&mut a.s[486], &a.v[73], &a.v[13]);
    k::metric(&mut a.s[487], &a.v[45], &a.v[0]);
    k::metric(&mut a.s[488], &a.v[19], &a.v[10]);
    k::metric(&mut a.s[489], &a.v[86], &a.v[7]);
    k::metric(&mut a.s[490], &a.v[60], &a.v[8]);
    k::metric(&mut a.s[491], &a.v[42], &a.v[7]);
    k::metric(&mut a.s[492], &a.v[5], &a.v[8]);
    k::metric(&mut a.s[493], &a.v[98], &a.v[7]);
    k::metric(&mut a.s[494], &a.v[85], &a.v[8]);
    k::metric(&mut a.s[495], &a.v[54], &a.v[7]);
    k::metric(&mut a.s[496], &a.v[41], &a.v[8]);
    k::metric(&mut a.s[497], &a.v[49], &a.v[3]);
    k::metric(&mut a.s[498], &a.v[27], &a.v[13]);
    k::metric(&mut a.s[499], &a.v[45], &a.v[1]);
    k::metric(&mut a.s[500], &a.v[19], &a.v[11]);
    k::metric(&mut a.s[501], &a.v[84], &a.v[7]);
    k::metric(&mut a.s[502], &a.v[80], &a.v[8]);
    k::metric(&mut a.s[503], &a.v[40], &a.v[7]);
    k::metric(&mut a.s[504], &a.v[36], &a.v[8]);
    k::metric(&mut a.s[505], &a.v[83], &a.v[7]);
    k::metric(&mut a.s[506], &a.v[75], &a.v[8]);
    k::metric(&mut a.s[507], &a.v[39], &a.v[7]);
    k::metric(&mut a.s[508], &a.v[31], &a.v[8]);
    k::metric(&mut a.s[509], &a.v[90], &a.v[3]);
    k::metric(&mut a.s[510], &a.v[67], &a.v[13]);
    k::metric(&mut a.s[511], &a.v[45], &a.v[2]);
    k::metric(&mut a.s[512], &a.v[19], &a.v[12]);
    k::metric(&mut a.s[513], &a.v[103], &a.v[4]);
    k::metric(&mut a.s[514], &a.v[79], &a.v[14]);
    k::metric(&mut a.s[515], &a.v[59], &a.v[4]);
    k::metric(&mut a.s[516], &a.v[35], &a.v[14]);
    k::metric(&mut a.s[517], &a.v[102], &a.v[4]);
    k::metric(&mut a.s[518], &a.v[78], &a.v[14]);
    k::metric(&mut a.s[519], &a.v[58], &a.v[4]);
    k::metric(&mut a.s[520], &a.v[34], &a.v[14]);
    k::metric(&mut a.s[521], &a.v[82], &a.v[15]);
    k::metric(&mut a.s[522], &a.v[70], &a.v[9]);
    k::metric(&mut a.s[523], &a.v[38], &a.v[6]);
    k::metric(&mut a.s[524], &a.v[26], &a.v[16]);
    k::metric(&mut a.s[525], &a.v[96], &a.v[4]);
    k::metric(&mut a.s[526], &a.v[76], &a.v[14]);
    k::metric(&mut a.s[527], &a.v[52], &a.v[4]);
    k::metric(&mut a.s[528], &a.v[32], &a.v[14]);
    k::metric(&mut a.s[529], &a.v[95], &a.v[4]);
    k::metric(&mut a.s[530], &a.v[74], &a.v[14]);
    k::metric(&mut a.s[531], &a.v[51], &a.v[4]);
    k::metric(&mut a.s[532], &a.v[30], &a.v[14]);
    k::metric(&mut a.s[533], &a.v[81], &a.v[15]);
    k::metric(&mut a.s[534], &a.v[64], &a.v[9]);
    k::metric(&mut a.s[535], &a.v[38], &a.v[0]);
    k::metric(&mut a.s[536], &a.v[26], &a.v[10]);
    k::metric(&mut a.s[537], &a.v[50], &a.v[4]);
    k::metric(&mut a.s[538], &a.v[29], &a.v[14]);
    k::metric(&mut a.s[539], &a.v[101], &a.v[4]);
    k::metric(&mut a.s[540], &a.v[72], &a.v[14]);
    k::metric(&mut a.s[541], &a.v[57], &a.v[4]);
    k::metric(&mut a.s[542], &a.v[28], &a.v[14]);
    k::metric(&mut a.s[543], &a.v[93], &a.v[4]);
    k::metric(&mut a.s[544], &a.v[71], &a.v[14]);
    k::metric(&mut a.s[545], &a.v[37], &a.v[15]);
    k::metric(&mut a.s[546], &a.v[20], &a.v[9]);
    k::metric(&mut a.s[547], &a.v[38], &a.v[1]);
    k::metric(&mut a.s[548], &a.v[26], &a.v[11]);
    k::metric(&mut a.s[549], &a.v[92], &a.v[4]);
    k::metric(&mut a.s[550], &a.v[69], &a.v[14]);
    k::metric(&mut a.s[551], &a.v[48], &a.v[4]);
    k::metric(&mut a.s[552], &a.v[25], &a.v[14]);
    k::metric(&mut a.s[553], &a.v[91], &a.v[4]);
    k::metric(&mut a.s[554], &a.v[68], &a.v[14]);
    k::metric(&mut a.s[555], &a.v[47], &a.v[4]);
    k::metric(&mut a.s[556], &a.v[24], &a.v[14]);
    k::metric(&mut a.s[557], &a.v[99], &a.v[15]);
    k::metric(&mut a.s[558], &a.v[65], &a.v[9]);
    k::metric(&mut a.s[559], &a.v[38], &a.v[2]);
    k::metric(&mut a.s[560], &a.v[26], &a.v[12]);
    k::metric(&mut a.s[561], &a.v[46], &a.v[4]);
    k::metric(&mut a.s[562], &a.v[23], &a.v[14]);
    k::metric(&mut a.s[563], &a.v[100], &a.v[4]);
    k::metric(&mut a.s[564], &a.v[66], &a.v[14]);
    k::metric(&mut a.s[565], &a.v[56], &a.v[4]);
    k::metric(&mut a.s[566], &a.v[22], &a.v[14]);
    k::metric(&mut a.s[567], &a.v[89], &a.v[4]);
    k::metric(&mut a.s[568], &a.v[63], &a.v[14]);
    k::metric(&mut a.s[569], &a.v[82], &a.v[3]);
    k::metric(&mut a.s[570], &a.v[70], &a.v[13]);
    k::metric(&mut a.s[571], &a.v[55], &a.v[6]);
    k::metric(&mut a.s[572], &a.v[21], &a.v[16]);
    k::metric(&mut a.s[573], &a.v[88], &a.v[4]);
    k::metric(&mut a.s[574], &a.v[62], &a.v[14]);
    k::metric(&mut a.s[575], &a.v[44], &a.v[4]);
    k::metric(&mut a.s[576], &a.v[18], &a.v[14]);
    k::metric(&mut a.s[577], &a.v[87], &a.v[4]);
    k::metric(&mut a.s[578], &a.v[61], &a.v[14]);
    k::metric(&mut a.s[579], &a.v[43], &a.v[4]);
    k::metric(&mut a.s[580], &a.v[17], &a.v[14]);
    k::metric(&mut a.s[581], &a.v[81], &a.v[3]);
    k::metric(&mut a.s[582], &a.v[64], &a.v[13]);
    k::metric(&mut a.s[583], &a.v[55], &a.v[0]);
    k::metric(&mut a.s[584], &a.v[21], &a.v[10]);
    k::metric(&mut a.s[585], &a.v[86], &a.v[4]);
    k::metric(&mut a.s[586], &a.v[60], &a.v[14]);
    k::metric(&mut a.s[587], &a.v[42], &a.v[4]);
    k::metric(&mut a.s[588], &a.v[5], &a.v[14]);
    k::metric(&mut a.s[589], &a.v[98], &a.v[4]);
    k::metric(&mut a.s[590], &a.v[85], &a.v[14]);
    k::metric(&mut a.s[591], &a.v[54], &a.v[4]);
    k::metric(&mut a.s[592], &a.v[41], &a.v[14]);
    k::metric(&mut a.s[593], &a.v[37], &a.v[3]);
    k::metric(&mut a.s[594], &a.v[20], &a.v[13]);
    k::metric(&mut a.s[595], &a.v[55], &a.v[1]);
    k::metric(&mut a.s[596], &a.v[21], &a.v[11]);
    k::metric(&mut a.s[597], &a.v[84], &a.v[4]);
    k::metric(&mut a.s[598], &a.v[80], &a.v[14]);
    k::metric(&mut a.s[599], &a.v[40], &a.v[4]);
    k::metric(&mut a.s[600], &a.v[36], &a.v[14]);
    k::metric(&mut a.s[601], &a.v[83], &a.v[4]);
    k::metric(&mut a.s[602], &a.v[75], &a.v[14]);
    k::metric(&mut a.s[603], &a.v[39], &a.v[4]);
    k::metric(&mut a.s[604], &a.v[31], &a.v[14]);
    k::metric(&mut a.s[605], &a.v[99], &a.v[3]);
    k::metric(&mut a.s[606], &a.v[65], &a.v[13]);
    k::metric(&mut a.s[607], &a.v[55], &a.v[2]);
    k::metric(&mut a.s[608], &a.v[21], &a.v[12]);
    { let (l, d, _) = sp(&mut a.s, 609); k::mul(d, &l[2], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 2); k::mul(d, &l[1], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 610); k::mul(d, &l[419], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 419); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 611); k::mul(d, &l[421], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 421); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 612); k::mul(d, &l[423], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 423); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 613); k::mul(d, &l[425], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 425); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 614); k::mul(d, &l[427], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 427); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 615); k::mul(d, &l[429], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 429); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 616); k::mul(d, &l[431], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 431); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 617); k::mul(d, &l[433], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 433); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 618); k::mul(d, &l[435], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 435); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 619); k::mul(d, &l[437], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 437); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 620); k::mul(d, &l[439], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 439); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 621); k::mul(d, &l[441], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 441); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 622); k::mul(d, &l[443], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 443); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 623); k::mul(d, &l[445], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 445); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 624); k::mul(d, &l[447], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 447); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 625); k::mul(d, &l[449], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 449); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 626); k::mul(d, &l[451], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 451); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 627); k::mul(d, &l[453], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 453); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 628); k::mul(d, &l[455], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 455); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 629); k::mul(d, &l[457], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 457); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 630); k::mul(d, &l[459], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 459); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 631); k::mul(d, &l[461], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 461); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 632); k::mul(d, &l[463], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 463); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 633); k::mul(d, &l[465], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 465); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 634); k::mul(d, &l[467], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 467); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 635); k::mul(d, &l[469], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 469); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 636); k::mul(d, &l[471], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 471); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 637); k::mul(d, &l[473], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 473); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 638); k::mul(d, &l[475], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 475); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 639); k::mul(d, &l[477], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 477); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 640); k::mul(d, &l[479], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 479); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 641); k::mul(d, &l[481], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 481); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 642); k::mul(d, &l[483], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 483); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 643); k::mul(d, &l[485], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 485); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 644); k::mul(d, &l[487], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 487); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 645); k::mul(d, &l[489], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 489); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 646); k::mul(d, &l[491], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 491); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 647); k::mul(d, &l[493], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 493); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 648); k::mul(d, &l[495], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 495); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 649); k::mul(d, &l[497], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 497); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 650); k::mul(d, &l[499], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 499); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 651); k::mul(d, &l[501], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 501); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 652); k::mul(d, &l[503], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 503); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 653); k::mul(d, &l[505], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 505); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 654); k::mul(d, &l[507], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 507); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 655); k::mul(d, &l[509], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 509); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 656); k::mul(d, &l[511], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 511); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 657); k::mul(d, &l[513], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 513); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 658); k::mul(d, &l[515], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 515); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 659); k::mul(d, &l[517], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 517); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 660); k::mul(d, &l[519], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 519); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 661); k::mul(d, &l[521], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 521); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 662); k::mul(d, &l[523], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 523); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 663); k::mul(d, &l[525], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 525); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 664); k::mul(d, &l[527], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 527); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 665); k::mul(d, &l[529], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 529); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 666); k::mul(d, &l[531], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 531); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 667); k::mul(d, &l[533], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 533); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 668); k::mul(d, &l[535], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 535); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 669); k::mul(d, &l[537], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 537); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 670); k::mul(d, &l[539], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 539); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 671); k::mul(d, &l[541], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 541); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 672); k::mul(d, &l[543], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 543); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 673); k::mul(d, &l[545], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 545); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 674); k::mul(d, &l[547], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 547); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 675); k::mul(d, &l[549], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 549); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 676); k::mul(d, &l[551], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 551); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 677); k::mul(d, &l[553], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 553); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 678); k::mul(d, &l[555], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 555); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 679); k::mul(d, &l[557], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 557); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 680); k::mul(d, &l[559], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 559); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 681); k::mul(d, &l[561], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 561); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 682); k::mul(d, &l[563], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 563); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 683); k::mul(d, &l[565], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 565); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 684); k::mul(d, &l[567], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 567); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 685); k::mul(d, &l[569], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 569); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 686); k::mul(d, &l[571], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 571); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 687); k::mul(d, &l[573], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 573); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 688); k::mul(d, &l[575], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 575); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 689); k::mul(d, &l[577], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 577); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 690); k::mul(d, &l[579], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 579); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 691); k::mul(d, &l[581], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 581); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 692); k::mul(d, &l[583], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 583); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 693); k::mul(d, &l[585], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 585); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 694); k::mul(d, &l[587], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 587); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 695); k::mul(d, &l[589], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 589); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 696); k::mul(d, &l[591], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 591); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 697); k::mul(d, &l[593], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 593); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 698); k::mul(d, &l[595], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 595); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 699); k::mul(d, &l[597], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 597); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 700); k::mul(d, &l[599], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 599); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 701); k::mul(d, &l[601], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 601); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 702); k::mul(d, &l[603], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 603); k::mul(d, &h[0], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 703); k::mul(d, &l[605], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 605); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 704); k::mul(d, &l[607], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 607); k::mul(d, &h[0], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 705); k::mul(d, &cc[0], &l[609]); }
    { let (_, d, h) = sp(&mut a.s, 609); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 610); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 611); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 612); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 613); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 614); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 615); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 616); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 617); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 618); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 619); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 620); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 621); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 622); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 623); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 624); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 625); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 626); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 627); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 628); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 629); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 630); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 631); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 632); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 633); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 634); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 635); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 636); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 637); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 638); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 639); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 640); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 641); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 642); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 643); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 644); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 645); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 646); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 647); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 648); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 649); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 650); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 651); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 652); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 653); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 654); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 655); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 656); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 657); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 658); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 659); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 660); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 661); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 662); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 663); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 664); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 665); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 666); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 667); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 668); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 669); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 670); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 671); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 672); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 673); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 674); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 675); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 676); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 677); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 678); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 679); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 680); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 681); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 682); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 683); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 684); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 685); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 686); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 687); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 688); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 689); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 690); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 691); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 692); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 693); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 694); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 695); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 696); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 697); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 698); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 699); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 700); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 701); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 702); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 703); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 704); k::mul(d, &h[0], &cr[3]); }
    { let (_, d, h) = sp(&mut a.s, 0); k::mul(d, &h[608], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 706); k::mul(d, &l[610], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 707); k::mul(d, &l[611], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 708); k::mul(d, &l[612], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 709); k::mul(d, &l[613], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 710); k::mul(d, &l[614], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 711); k::mul(d, &l[615], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 712); k::mul(d, &l[616], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 713); k::mul(d, &l[617], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 714); k::mul(d, &l[618], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 715); k::mul(d, &l[619], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 716); k::mul(d, &l[620], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 717); k::mul(d, &l[621], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 718); k::mul(d, &l[622], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 719); k::mul(d, &l[623], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 720); k::mul(d, &l[624], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 721); k::mul(d, &l[625], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 722); k::mul(d, &l[626], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 723); k::mul(d, &l[627], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 724); k::mul(d, &l[628], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 725); k::mul(d, &l[629], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 726); k::mul(d, &l[630], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 727); k::mul(d, &l[631], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 728); k::mul(d, &l[632], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 729); k::mul(d, &l[633], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 730); k::mul(d, &l[634], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 731); k::mul(d, &l[635], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 732); k::mul(d, &l[636], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 733); k::mul(d, &l[637], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 734); k::mul(d, &l[638], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 735); k::mul(d, &l[639], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 736); k::mul(d, &l[640], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 737); k::mul(d, &l[641], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 738); k::mul(d, &l[642], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 739); k::mul(d, &l[643], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 740); k::mul(d, &l[644], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 741); k::mul(d, &l[645], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 742); k::mul(d, &l[646], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 743); k::mul(d, &l[647], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 744); k::mul(d, &l[648], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 745); k::mul(d, &l[649], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 746); k::mul(d, &l[650], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 747); k::mul(d, &l[651], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 748); k::mul(d, &l[652], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 749); k::mul(d, &l[653], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 750); k::mul(d, &l[654], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 751); k::mul(d, &l[655], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 752); k::mul(d, &l[656], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 753); k::mul(d, &l[657], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 754); k::mul(d, &l[658], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 755); k::mul(d, &l[659], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 756); k::mul(d, &l[660], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 757); k::mul(d, &l[661], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 758); k::mul(d, &l[662], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 759); k::mul(d, &l[663], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 760); k::mul(d, &l[664], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 761); k::mul(d, &l[665], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 762); k::mul(d, &l[666], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 763); k::mul(d, &l[667], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 764); k::mul(d, &l[668], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 765); k::mul(d, &l[669], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 766); k::mul(d, &l[670], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 767); k::mul(d, &l[671], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 768); k::mul(d, &l[672], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 769); k::mul(d, &l[673], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 770); k::mul(d, &l[674], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 771); k::mul(d, &l[675], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 772); k::mul(d, &l[676], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 773); k::mul(d, &l[677], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 774); k::mul(d, &l[678], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 775); k::mul(d, &l[679], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 776); k::mul(d, &l[680], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 777); k::mul(d, &l[681], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 778); k::mul(d, &l[682], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 779); k::mul(d, &l[683], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 780); k::mul(d, &l[684], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 781); k::mul(d, &l[685], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 782); k::mul(d, &l[686], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 783); k::mul(d, &l[687], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 784); k::mul(d, &l[688], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 785); k::mul(d, &l[689], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 786); k::mul(d, &l[690], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 787); k::mul(d, &l[691], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 788); k::mul(d, &l[692], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 789); k::mul(d, &l[693], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 790); k::mul(d, &l[694], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 791); k::mul(d, &l[695], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 792); k::mul(d, &l[696], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 793); k::mul(d, &l[697], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 794); k::mul(d, &l[698], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 795); k::mul(d, &l[699], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 796); k::mul(d, &l[700], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 797); k::mul(d, &l[701], &cr[3]); }
    { let (l, d, _) = sp(&mut a.s, 798); k::mul(d, &l[702], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 799); k::mul(d, &l[703], &cr[19]); }
    { let (l, d, _) = sp(&mut a.s, 800); k::add(d, &[&l[704], &l[307], &l[2], &l[6], &l[0], &l[4], &l[419], &l[22], &l[706], &l[309], &l[421], &l[24], &l[707], &l[310], &l[423], &l[26], &l[311], &l[708], &l[312], &l[425], &l[28], &l[709], &l[313], &l[427], &l[30]]); }
    { let (_, d, h) = sp(&mut a.s, 30); k::add(d, &[&h[679], &h[283], &h[398], &h[1], &h[680], &h[284], &h[400], &h[3], &h[681], &h[285], &h[402], &h[5], &h[682], &h[286], &h[404], &h[7], &h[287], &h[683], &h[288], &h[406], &h[9], &h[684], &h[289], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 42); k::add(d, &[&h[673], &h[278], &h[398], &h[1], &h[674], &h[279], &h[400], &h[3], &h[675], &h[280], &h[402], &h[5], &h[676], &h[281], &h[404], &h[7], &h[282], &h[677], &h[283], &h[406], &h[9], &h[678], &h[284], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 54); k::add(d, &[&h[667], &h[273], &h[398], &h[1], &h[668], &h[274], &h[400], &h[3], &h[669], &h[275], &h[402], &h[5], &h[670], &h[276], &h[404], &h[7], &h[277], &h[671], &h[278], &h[406], &h[9], &h[672], &h[279], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 66); k::add(d, &[&h[661], &h[268], &h[398], &h[1], &h[662], &h[269], &h[400], &h[3], &h[663], &h[270], &h[402], &h[5], &h[664], &h[271], &h[404], &h[7], &h[272], &h[665], &h[273], &h[406], &h[9], &h[666], &h[274], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 78); k::add(d, &[&h[655], &h[263], &h[398], &h[1], &h[656], &h[264], &h[400], &h[3], &h[657], &h[265], &h[402], &h[5], &h[658], &h[266], &h[404], &h[7], &h[267], &h[659], &h[268], &h[406], &h[9], &h[660], &h[269], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 90); k::add(d, &[&h[649], &h[258], &h[398], &h[1], &h[650], &h[259], &h[400], &h[3], &h[651], &h[260], &h[402], &h[5], &h[652], &h[261], &h[404], &h[7], &h[262], &h[653], &h[263], &h[406], &h[9], &h[654], &h[264], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 102); k::add(d, &[&h[643], &h[253], &h[398], &h[1], &h[644], &h[254], &h[400], &h[3], &h[645], &h[255], &h[402], &h[5], &h[646], &h[256], &h[404], &h[7], &h[257], &h[647], &h[258], &h[406], &h[9], &h[648], &h[259], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 114); k::add(d, &[&h[637], &h[248], &h[398], &h[1], &h[638], &h[249], &h[400], &h[3], &h[639], &h[250], &h[402], &h[5], &h[640], &h[251], &h[404], &h[7], &h[252], &h[641], &h[253], &h[406], &h[9], &h[642], &h[254], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 126); k::add(d, &[&h[631], &h[243], &h[398], &h[1], &h[632], &h[244], &h[400], &h[3], &h[633], &h[245], &h[402], &h[5], &h[634], &h[246], &h[404], &h[7], &h[247], &h[635], &h[248], &h[406], &h[9], &h[636], &h[249], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 138); k::add(d, &[&h[625], &h[238], &h[398], &h[1], &h[626], &h[239], &h[400], &h[3], &h[627], &h[240], &h[402], &h[5], &h[628], &h[241], &h[404], &h[7], &h[242], &h[629], &h[243], &h[406], &h[9], &h[630], &h[244], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 150); k::add(d, &[&h[619], &h[233], &h[398], &h[1], &h[620], &h[234], &h[400], &h[3], &h[621], &h[235], &h[402], &h[5], &h[622], &h[236], &h[404], &h[7], &h[237], &h[623], &h[238], &h[406], &h[9], &h[624], &h[239], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 162); k::add(d, &[&h[613], &h[228], &h[398], &h[1], &h[614], &h[229], &h[400], &h[3], &h[615], &h[230], &h[402], &h[5], &h[616], &h[231], &h[404], &h[7], &h[232], &h[617], &h[233], &h[406], &h[9], &h[618], &h[234], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 174); k::add(d, &[&h[607], &h[223], &h[398], &h[1], &h[608], &h[224], &h[400], &h[3], &h[609], &h[225], &h[402], &h[5], &h[610], &h[226], &h[404], &h[7], &h[227], &h[611], &h[228], &h[406], &h[9], &h[612], &h[229], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 186); k::add(d, &[&h[601], &h[218], &h[398], &h[1], &h[602], &h[219], &h[400], &h[3], &h[603], &h[220], &h[402], &h[5], &h[604], &h[221], &h[404], &h[7], &h[222], &h[605], &h[223], &h[406], &h[9], &h[606], &h[224], &h[408], &h[11]]); }
    { let (_, d, h) = sp(&mut a.s, 198); k::add(d, &[&h[595], &h[213], &h[398], &h[1], &h[596], &h[214], &h[400], &h[3], &h[597], &h[215], &h[402], &h[5], &h[598], &h[216], &h[404], &h[7], &h[217], &h[599], &h[218], &h[406], &h[9], &h[600], &h[219], &h[408], &h[11]]); }
}
