// chunk: 0. Rendered from the helicity-expanded program of `ee_to_mumu` (66 instructions, 52 calls, 1 function(s), slots per class [6, 17, 9, 0, 4, 4]); regenerate, do not edit.
pub(super) fn bind_ee_to_mumu<F: Real>() -> Box<dyn MgRun<F>> {
    bound::<F, 17, 9, 0, 4, 4>(mg_ee_to_mumu::<F>)
}
#[inline(never)]
fn mg_ee_to_mumu<F: Real>(a: &mut MgArenas<F, 17, 9, 0, 4, 4>, mo: &[LorentzVector<F>], cc: &[C<F>], cr: &[F], mm: &[LorentzVector<F>]) {
    let mo: &[LorentzVector<F>; 4] = mo.try_into().expect("external momenta");
    let cc: &[C<F>; 5] = cc.try_into().expect("complex pool");
    let cr: &[F; 11] = cr.try_into().expect("real pool");
    let mm: &[LorentzVector<F>; 8] = mm.try_into().expect("momentum pool");
    k::ext_fin(&mut a.i[0], &mo[1], 1, 2, Charge::Particle, true, &cr[1]);
    k::ext_fin(&mut a.i[1], &mo[3], 1, 2, Charge::Antiparticle, false, &cr[2]);
    k::ext_fin(&mut a.i[2], &mo[3], -1, 2, Charge::Antiparticle, false, &cr[2]);
    k::ext_fin(&mut a.i[3], &mo[1], -1, 2, Charge::Particle, true, &cr[1]);
    k::ext_fout(&mut a.o[0], &mo[0], -1, 2, Charge::Antiparticle, true, &cr[0]);
    k::ext_fout(&mut a.o[1], &mo[2], -1, 2, Charge::Particle, false, &cr[3]);
    k::ext_fout(&mut a.o[2], &mo[2], 1, 2, Charge::Particle, false, &cr[3]);
    k::ext_fout(&mut a.o[3], &mo[0], 1, 2, Charge::Antiparticle, true, &cr[0]);
    k::gamma_vout(&mut a.v[0], &a.o[1], &a.i[1], false);
    k::gamma_vout(&mut a.v[1], &a.o[0], &a.i[0], true);
    k::gamma_vout(&mut a.v[2], &a.o[2], &a.i[2], false);
    k::gamma_vout(&mut a.v[3], &a.o[3], &a.i[3], true);
    k::ffv_vout(&mut a.v[4], &a.o[1], &a.i[1], &cc[3], &cc[4], false);
    k::ffv_vout(&mut a.v[5], &a.o[0], &a.i[0], &cc[3], &cc[4], true);
    k::ffv_vout(&mut a.v[6], &a.o[2], &a.i[2], &cc[3], &cc[4], false);
    k::ffv_vout(&mut a.v[7], &a.o[3], &a.i[3], &cc[3], &cc[4], true);
    { let (l, d, _) = sp(&mut a.v, 8); k::propagate_vector(d, &l[4], &mm[5], &cr[9], &cr[10]); }
    { let (_, d, h) = sp(&mut a.v, 4); k::propagate_vector(d, &h[1], &mm[5], &cr[9], &cr[10]); }
    { let (l, d, _) = sp(&mut a.v, 6); k::mul(d, &l[0], &cr[4]); }
    { let (_, d, h) = sp(&mut a.v, 0); k::mul(d, &h[1], &cr[4]); }
    { let (_, d, h) = sp(&mut a.v, 2); k::mul(d, &h[3], &cc[0]); }
    { let (l, d, _) = sp(&mut a.v, 6); k::mul(d, &l[0], &cc[0]); }
    k::metric(&mut a.s[2], &a.v[5], &a.v[8]);
    k::metric(&mut a.s[1], &a.v[5], &a.v[4]);
    k::metric(&mut a.s[3], &a.v[7], &a.v[8]);
    k::metric(&mut a.s[4], &a.v[7], &a.v[4]);
    { let (l, d, _) = sp(&mut a.v, 7); k::propagate_vector(d, &l[2], &mm[5], &cr[5], &cr[6]); }
    { let (_, d, h) = sp(&mut a.v, 2); k::propagate_vector(d, &h[3], &mm[5], &cr[5], &cr[6]); }
    { let (l, d, _) = sp(&mut a.s, 5); k::mul(d, &l[2], &cr[7]); }
    { let (l, d, _) = sp(&mut a.s, 6); k::mul(d, &l[1], &cr[7]); }
    { let (l, d, _) = sp(&mut a.s, 7); k::mul(d, &l[3], &cr[7]); }
    { let (l, d, _) = sp(&mut a.s, 8); k::mul(d, &l[4], &cr[7]); }
    k::metric(&mut a.s[9], &a.v[1], &a.v[7]);
    k::metric(&mut a.s[10], &a.v[1], &a.v[2]);
    k::metric(&mut a.s[11], &a.v[3], &a.v[7]);
    k::metric(&mut a.s[12], &a.v[3], &a.v[2]);
    { let (l, d, _) = sp(&mut a.s, 13); k::mul(d, &l[9], &cr[4]); }
    { let (_, d, h) = sp(&mut a.s, 9); k::mul(d, &h[0], &cr[4]); }
    { let (_, d, h) = sp(&mut a.s, 10); k::mul(d, &h[0], &cr[4]); }
    { let (_, d, h) = sp(&mut a.s, 11); k::mul(d, &h[0], &cr[4]); }
    { let (_, d, h) = sp(&mut a.s, 12); k::mul(d, &cc[0], &h[0]); }
    { let (l, d, _) = sp(&mut a.s, 13); k::mul(d, &cc[0], &l[9]); }
    { let (_, d, h) = sp(&mut a.s, 9); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 10); k::mul(d, &cc[0], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 11); k::mul(d, &h[0], &cr[7]); }
    { let (_, d, h) = sp(&mut a.s, 0); k::mul(d, &h[12], &cr[7]); }
    { let (l, d, _) = sp(&mut a.s, 14); k::mul(d, &l[9], &cr[7]); }
    { let (l, d, _) = sp(&mut a.s, 15); k::mul(d, &l[10], &cr[7]); }
    { let (l, d, _) = sp(&mut a.s, 16); k::add(d, &[&l[11], &l[5]]); }
    { let (l, d, h) = sp(&mut a.s, 5); k::add(d, &[&l[0], &h[0]]); }
    { let (_, d, h) = sp(&mut a.s, 6); k::add(d, &[&h[7], &h[0]]); }
    { let (_, d, h) = sp(&mut a.s, 7); k::add(d, &[&h[7], &h[0]]); }
}
