// chunk: 0. Rendered from the helicity-expanded program of `gg_to_gg` (695 instructions, 673 calls, 1 function(s), slots per class [5, 132, 85, 0, 0, 0]); regenerate, do not edit.
pub(super) fn bind_gg_to_gg<F: Real>() -> Box<dyn MgRun<F>> {
    bound::<F, 132, 85, 0, 0, 0>(mg_gg_to_gg::<F>)
}
#[inline(never)]
fn mg_gg_to_gg<F: Real>(a: &mut MgArenas<F, 132, 85, 0, 0, 0>, mo: &[LorentzVector<F>], cc: &[C<F>], cr: &[F], mm: &[LorentzVector<F>]) {
    let mo: &[LorentzVector<F>; 4] = mo.try_into().expect("external momenta");
    let cc: &[C<F>; 4] = cc.try_into().expect("complex pool");
    let cr: &[F; 7] = cr.try_into().expect("real pool");
    let mm: &[LorentzVector<F>; 16] = mm.try_into().expect("momentum pool");
    k::ext_vector(&mut a.v[0], &mo[0], -1, 3, Charge::Particle, true, &cr[0]);
    k::ext_vector(&mut a.v[1], &mo[1], -1, 3, Charge::Particle, true, &cr[0]);
    k::ext_vector(&mut a.v[2], &mo[2], -1, 3, Charge::Particle, false, &cr[0]);
    k::ext_vector(&mut a.v[3], &mo[3], -1, 3, Charge::Particle, false, &cr[0]);
    k::ext_vector(&mut a.v[4], &mo[1], 1, 3, Charge::Particle, true, &cr[0]);
    k::ext_vector(&mut a.v[5], &mo[3], 1, 3, Charge::Particle, false, &cr[0]);
    k::ext_vector(&mut a.v[6], &mo[2], 1, 3, Charge::Particle, false, &cr[0]);
    k::ext_vector(&mut a.v[7], &mo[0], 1, 3, Charge::Particle, true, &cr[0]);
    k::metric(&mut a.s[4], &a.v[0], &a.v[3]);
    k::metric(&mut a.s[5], &a.v[1], &a.v[2]);
    k::metric(&mut a.s[6], &a.v[0], &a.v[2]);
    k::metric(&mut a.s[7], &a.v[1], &a.v[3]);
    k::metric(&mut a.s[8], &a.v[0], &a.v[1]);
    k::metric(&mut a.s[9], &a.v[2], &a.v[3]);
    k::metric(&mut a.s[10], &a.v[0], &a.v[5]);
    k::metric(&mut a.s[11], &a.v[4], &a.v[2]);
    k::metric(&mut a.s[12], &a.v[4], &a.v[5]);
    k::metric(&mut a.s[13], &a.v[2], &a.v[5]);
    k::metric(&mut a.s[14], &a.v[4], &a.v[6]);
    k::metric(&mut a.s[15], &a.v[0], &a.v[6]);
    k::metric(&mut a.s[16], &a.v[4], &a.v[3]);
    k::metric(&mut a.s[17], &a.v[6], &a.v[3]);
    k::metric(&mut a.s[18], &a.v[7], &a.v[5]);
    k::metric(&mut a.s[19], &a.v[7], &a.v[2]);
    k::metric(&mut a.s[20], &a.v[1], &a.v[5]);
    k::metric(&mut a.s[21], &a.v[7], &a.v[3]);
    k::metric(&mut a.s[22], &a.v[1], &a.v[6]);
    k::metric(&mut a.s[23], &a.v[7], &a.v[6]);
    k::metric(&mut a.s[24], &a.v[7], &a.v[4]);
    k::metric(&mut a.s[25], &a.v[6], &a.v[5]);
    { let (l, d, _) = sp(&mut a.v, 8); k::metric_vout(d, &l[2]); }
    { let (l, d, _) = sp(&mut a.v, 9); k::metric_vout(d, &l[3]); }
    { let (l, d, _) = sp(&mut a.v, 10); k::metric_vout(d, &l[1]); }
    { let (l, d, _) = sp(&mut a.v, 11); k::metric_vout(d, &l[5]); }
    { let (l, d, _) = sp(&mut a.v, 12); k::metric_vout(d, &l[4]); }
    { let (l, d, _) = sp(&mut a.v, 13); k::metric_vout(d, &l[6]); }
    k::pmom(&mut a.v[14], &mm[3]);
    k::pmom(&mut a.v[15], &mm[4]);
    k::pmom(&mut a.v[16], &mm[1]);
    k::pmom(&mut a.v[17], &mm[2]);
    k::pmom_out(&mut a.v[18], mm, &[(3, 1), (4, 1)]);
    k::pmom_out(&mut a.v[19], mm, &[(2, 1), (3, 1)]);
    k::pmom_out(&mut a.v[20], mm, &[(2, 1), (4, 1)]);
    { let (l, d, _) = sp(&mut a.s, 26); k::mul(d, &l[4], &l[5]); }
    { let (l, d, _) = sp(&mut a.s, 27); k::mul(d, &l[6], &l[7]); }
    { let (l, d, _) = sp(&mut a.s, 28); k::mul(d, &l[8], &l[9]); }
    { let (l, d, _) = sp(&mut a.s, 29); k::mul(d, &l[10], &l[11]); }
    { let (l, d, _) = sp(&mut a.s, 30); k::mul(d, &l[6], &l[12]); }
    { let (l, d, _) = sp(&mut a.s, 31); k::mul(d, &l[4], &l[14]); }
    { let (l, d, _) = sp(&mut a.s, 32); k::mul(d, &l[15], &l[16]); }
    { let (l, d, _) = sp(&mut a.s, 33); k::mul(d, &l[18], &l[5]); }
    { let (l, d, _) = sp(&mut a.s, 34); k::mul(d, &l[19], &l[20]); }
    { let (l, d, _) = sp(&mut a.s, 35); k::mul(d, &l[21], &l[22]); }
    { let (l, d, _) = sp(&mut a.s, 36); k::mul(d, &l[23], &l[7]); }
    { let (l, d, _) = sp(&mut a.s, 37); k::mul(d, &l[18], &l[14]); }
    { let (l, d, _) = sp(&mut a.s, 38); k::mul(d, &l[23], &l[12]); }
    { let (l, d, _) = sp(&mut a.s, 39); k::mul(d, &l[24], &l[25]); }
    k::metric(&mut a.s[40], &a.v[14], &a.v[3]);
    k::metric(&mut a.s[41], &a.v[18], &a.v[3]);
    k::metric(&mut a.s[42], &a.v[15], &a.v[2]);
    k::metric(&mut a.s[43], &a.v[18], &a.v[2]);
    k::metric(&mut a.s[44], &a.v[17], &a.v[2]);
    k::metric(&mut a.s[45], &a.v[19], &a.v[2]);
    k::metric(&mut a.s[46], &a.v[14], &a.v[1]);
    k::metric(&mut a.s[47], &a.v[19], &a.v[1]);
    k::metric(&mut a.s[48], &a.v[16], &a.v[3]);
    k::metric(&mut a.s[49], &a.v[15], &a.v[0]);
    k::metric(&mut a.s[50], &a.v[17], &a.v[3]);
    k::metric(&mut a.s[51], &a.v[20], &a.v[3]);
    k::metric(&mut a.s[52], &a.v[15], &a.v[1]);
    k::metric(&mut a.s[53], &a.v[20], &a.v[1]);
    k::metric(&mut a.s[54], &a.v[16], &a.v[2]);
    k::metric(&mut a.s[55], &a.v[14], &a.v[0]);
    k::metric(&mut a.s[56], &a.v[14], &a.v[5]);
    k::metric(&mut a.s[57], &a.v[18], &a.v[5]);
    k::metric(&mut a.s[58], &a.v[14], &a.v[4]);
    k::metric(&mut a.s[59], &a.v[19], &a.v[4]);
    k::metric(&mut a.s[60], &a.v[16], &a.v[5]);
    k::metric(&mut a.s[61], &a.v[17], &a.v[5]);
    k::metric(&mut a.s[62], &a.v[20], &a.v[5]);
    k::metric(&mut a.s[63], &a.v[15], &a.v[4]);
    k::metric(&mut a.s[64], &a.v[20], &a.v[4]);
    k::metric(&mut a.s[65], &a.v[15], &a.v[6]);
    k::metric(&mut a.s[66], &a.v[18], &a.v[6]);
    k::metric(&mut a.s[67], &a.v[17], &a.v[6]);
    k::metric(&mut a.s[68], &a.v[19], &a.v[6]);
    k::metric(&mut a.s[69], &a.v[16], &a.v[6]);
    k::metric(&mut a.s[70], &a.v[15], &a.v[7]);
    k::metric(&mut a.s[71], &a.v[14], &a.v[7]);
    { let (l, d, _) = sp(&mut a.v, 19); k::metric_vout(d, &l[14]); }
    { let (l, d, _) = sp(&mut a.v, 18); k::metric_vout(d, &l[15]); }
    { let (l, d, _) = sp(&mut a.v, 20); k::metric_vout(d, &l[17]); }
    { let (l, d, _) = sp(&mut a.s, 72); k::mul(d, &l[26], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 26); k::mul(d, &h[0], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 73); k::mul(d, &l[27], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 27); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 28); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 29); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 30); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 31); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 32); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 33); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 34); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 35); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 36); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 37); k::mul(d, &h[0], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 74); k::mul(d, &l[38], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 38); k::mul(d, &h[0], &cr[2]); }
    { let (l, d, _) = sp(&mut a.v, 21); k::mul(d, &l[19], &a.s[9]); }
    { let (l, d, _) = sp(&mut a.v, 22); k::mul(d, &l[18], &a.s[9]); }
    { let (l, d, _) = sp(&mut a.v, 23); k::mul(d, &l[8], &a.s[40]); }
    { let (l, d, _) = sp(&mut a.v, 24); k::mul(d, &l[8], &a.s[41]); }
    { let (l, d, _) = sp(&mut a.v, 25); k::mul(d, &l[9], &a.s[42]); }
    { let (l, d, _) = sp(&mut a.v, 26); k::mul(d, &l[9], &a.s[43]); }
    { let (l, d, _) = sp(&mut a.v, 27); k::mul(d, &l[20], &a.s[5]); }
    { let (l, d, _) = sp(&mut a.v, 28); k::mul(d, &l[19], &a.s[5]); }
    { let (l, d, _) = sp(&mut a.v, 29); k::mul(d, &l[10], &a.s[44]); }
    { let (l, d, _) = sp(&mut a.v, 30); k::mul(d, &l[10], &a.s[45]); }
    { let (l, d, _) = sp(&mut a.v, 31); k::mul(d, &l[8], &a.s[46]); }
    { let (l, d, _) = sp(&mut a.v, 32); k::mul(d, &l[8], &a.s[47]); }
    { let (l, d, _) = sp(&mut a.v, 33); k::mul(d, &l[20], &a.s[7]); }
    { let (l, d, _) = sp(&mut a.v, 34); k::mul(d, &l[18], &a.s[7]); }
    { let (l, d, _) = sp(&mut a.v, 35); k::mul(d, &l[10], &a.s[50]); }
    { let (l, d, _) = sp(&mut a.v, 36); k::mul(d, &l[10], &a.s[51]); }
    { let (l, d, _) = sp(&mut a.v, 37); k::mul(d, &l[9], &a.s[52]); }
    { let (l, d, _) = sp(&mut a.v, 38); k::mul(d, &l[9], &a.s[53]); }
    { let (l, d, _) = sp(&mut a.v, 39); k::mul(d, &l[19], &a.s[13]); }
    { let (l, d, _) = sp(&mut a.v, 40); k::mul(d, &l[18], &a.s[13]); }
    { let (l, d, _) = sp(&mut a.v, 41); k::mul(d, &l[8], &a.s[56]); }
    { let (l, d, _) = sp(&mut a.v, 42); k::mul(d, &l[8], &a.s[57]); }
    { let (l, d, _) = sp(&mut a.v, 43); k::mul(d, &l[11], &a.s[42]); }
    { let (l, d, _) = sp(&mut a.v, 44); k::mul(d, &l[11], &a.s[43]); }
    { let (l, d, _) = sp(&mut a.v, 45); k::mul(d, &l[20], &a.s[11]); }
    { let (l, d, _) = sp(&mut a.v, 46); k::mul(d, &l[19], &a.s[11]); }
    { let (l, d, _) = sp(&mut a.v, 47); k::mul(d, &l[12], &a.s[44]); }
    { let (l, d, _) = sp(&mut a.v, 48); k::mul(d, &l[12], &a.s[45]); }
    { let (l, d, _) = sp(&mut a.v, 49); k::mul(d, &l[8], &a.s[58]); }
    { let (l, d, _) = sp(&mut a.v, 50); k::mul(d, &l[8], &a.s[59]); }
    { let (_, d, h) = sp(&mut a.v, 8); k::mul(d, &h[11], &a.s[12]); }
    { let (l, d, _) = sp(&mut a.v, 51); k::mul(d, &l[18], &a.s[12]); }
    { let (l, d, _) = sp(&mut a.v, 52); k::mul(d, &l[12], &a.s[61]); }
    { let (l, d, _) = sp(&mut a.v, 53); k::mul(d, &l[12], &a.s[62]); }
    { let (l, d, _) = sp(&mut a.v, 54); k::mul(d, &l[11], &a.s[63]); }
    { let (l, d, _) = sp(&mut a.v, 55); k::mul(d, &l[11], &a.s[64]); }
    { let (l, d, _) = sp(&mut a.v, 56); k::mul(d, &l[19], &a.s[17]); }
    { let (l, d, _) = sp(&mut a.v, 57); k::mul(d, &l[18], &a.s[17]); }
    { let (l, d, _) = sp(&mut a.v, 58); k::mul(d, &l[13], &a.s[40]); }
    { let (l, d, _) = sp(&mut a.v, 59); k::mul(d, &l[13], &a.s[41]); }
    { let (l, d, _) = sp(&mut a.v, 60); k::mul(d, &l[9], &a.s[65]); }
    { let (l, d, _) = sp(&mut a.v, 61); k::mul(d, &l[9], &a.s[66]); }
    { let (l, d, _) = sp(&mut a.v, 62); k::mul(d, &l[20], &a.s[14]); }
    { let (l, d, _) = sp(&mut a.v, 63); k::mul(d, &l[19], &a.s[14]); }
    { let (l, d, _) = sp(&mut a.v, 64); k::mul(d, &l[12], &a.s[67]); }
    { let (l, d, _) = sp(&mut a.v, 65); k::mul(d, &l[12], &a.s[68]); }
    { let (l, d, _) = sp(&mut a.v, 66); k::mul(d, &l[13], &a.s[58]); }
    { let (l, d, _) = sp(&mut a.v, 67); k::mul(d, &l[13], &a.s[59]); }
    { let (l, d, _) = sp(&mut a.v, 68); k::mul(d, &l[20], &a.s[16]); }
    { let (l, d, _) = sp(&mut a.v, 69); k::mul(d, &l[18], &a.s[16]); }
    { let (l, d, _) = sp(&mut a.v, 70); k::mul(d, &l[12], &a.s[50]); }
    { let (l, d, _) = sp(&mut a.v, 71); k::mul(d, &l[12], &a.s[51]); }
    { let (l, d, _) = sp(&mut a.v, 12); k::mul(d, &l[9], &a.s[63]); }
    { let (l, d, _) = sp(&mut a.v, 72); k::mul(d, &l[9], &a.s[64]); }
    { let (_, d, h) = sp(&mut a.v, 9); k::mul(d, &h[10], &a.s[20]); }
    { let (l, d, _) = sp(&mut a.v, 73); k::mul(d, &l[18], &a.s[20]); }
    { let (l, d, _) = sp(&mut a.v, 74); k::mul(d, &l[10], &a.s[61]); }
    { let (l, d, _) = sp(&mut a.v, 75); k::mul(d, &l[10], &a.s[62]); }
    { let (l, d, _) = sp(&mut a.v, 76); k::mul(d, &l[11], &a.s[52]); }
    { let (l, d, _) = sp(&mut a.v, 77); k::mul(d, &l[11], &a.s[53]); }
    { let (l, d, _) = sp(&mut a.v, 78); k::mul(d, &l[20], &a.s[22]); }
    { let (l, d, _) = sp(&mut a.v, 20); k::mul(d, &l[19], &a.s[22]); }
    { let (l, d, _) = sp(&mut a.v, 79); k::mul(d, &l[10], &a.s[67]); }
    { let (l, d, _) = sp(&mut a.v, 80); k::mul(d, &l[10], &a.s[68]); }
    { let (_, d, h) = sp(&mut a.v, 10); k::mul(d, &h[2], &a.s[46]); }
    { let (l, d, _) = sp(&mut a.v, 81); k::mul(d, &l[13], &a.s[47]); }
    { let (l, d, _) = sp(&mut a.v, 82); k::mul(d, &l[19], &a.s[25]); }
    { let (l, d, _) = sp(&mut a.v, 19); k::mul(d, &l[18], &a.s[25]); }
    { let (l, d, _) = sp(&mut a.v, 18); k::mul(d, &l[13], &a.s[56]); }
    { let (l, d, _) = sp(&mut a.v, 83); k::mul(d, &l[13], &a.s[57]); }
    { let (l, d, _) = sp(&mut a.v, 13); k::mul(d, &l[11], &a.s[65]); }
    { let (l, d, _) = sp(&mut a.v, 84); k::mul(d, &l[11], &a.s[66]); }
    { let (l, d, h) = sp(&mut a.s, 66); k::add(d, &[&h[5], &l[26]]); }
    { let (_, d, h) = sp(&mut a.s, 26); k::add(d, &[&h[46], &h[0]]); }
    { let (l, d, _) = sp(&mut a.s, 73); k::add(d, &[&l[72], &l[27]]); }
    { let (_, d, h) = sp(&mut a.s, 27); k::add(d, &[&h[1]]); }
    { let (l, d, _) = sp(&mut a.s, 29); k::add(d, &[&l[28]]); }
    { let (_, d, h) = sp(&mut a.s, 28); k::add(d, &[&h[2]]); }
    { let (l, d, _) = sp(&mut a.s, 31); k::add(d, &[&l[30]]); }
    { let (_, d, h) = sp(&mut a.s, 30); k::add(d, &[&h[2]]); }
    { let (l, d, _) = sp(&mut a.s, 33); k::add(d, &[&l[32]]); }
    { let (_, d, h) = sp(&mut a.s, 32); k::add(d, &[&h[2]]); }
    { let (l, d, _) = sp(&mut a.s, 35); k::add(d, &[&l[34]]); }
    { let (_, d, h) = sp(&mut a.s, 34); k::add(d, &[&h[1], &h[2]]); }
    { let (_, d, h) = sp(&mut a.s, 37); k::add(d, &[&h[36], &h[0]]); }
    { let (l, d, _) = sp(&mut a.s, 74); k::add(d, &[&l[36], &l[38]]); }
    { let (_, d, h) = sp(&mut a.v, 11); k::mul(d, &h[9], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 21); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 22); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 23); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 24); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 25); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 26); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 27); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 28); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 29); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 30); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 31); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 32); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 33); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 34); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 35); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 36); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 37); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 38); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 39); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 40); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 41); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 42); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 43); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 44); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 45); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 46); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 47); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 48); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 49); k::mul(d, &h[0], &cr[2]); }
    { let (l, d, _) = sp(&mut a.v, 50); k::mul(d, &l[8], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 8); k::mul(d, &h[42], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 51); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 52); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 53); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 54); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 55); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 56); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 57); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 58); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 59); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 60); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 61); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 62); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 63); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 64); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 65); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 66); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 67); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 68); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 69); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 70); k::mul(d, &h[0], &cr[1]); }
    { let (l, d, _) = sp(&mut a.v, 71); k::mul(d, &l[12], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 12); k::mul(d, &h[59], &cr[2]); }
    { let (l, d, _) = sp(&mut a.v, 72); k::mul(d, &l[9], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 9); k::mul(d, &h[63], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 73); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 74); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 75); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 76); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 77); k::mul(d, &h[0], &cr[1]); }
    { let (l, d, _) = sp(&mut a.v, 78); k::mul(d, &l[20], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 20); k::mul(d, &h[58], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 79); k::mul(d, &h[0], &cr[1]); }
    { let (l, d, _) = sp(&mut a.v, 80); k::mul(d, &l[10], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 10); k::mul(d, &h[70], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 81); k::mul(d, &h[0], &cr[1]); }
    { let (l, d, _) = sp(&mut a.v, 82); k::mul(d, &l[19], &cr[2]); }
    { let (l, d, _) = sp(&mut a.v, 19); k::mul(d, &l[18], &cr[2]); }
    { let (_, d, h) = sp(&mut a.v, 18); k::mul(d, &h[64], &cr[1]); }
    { let (l, d, _) = sp(&mut a.v, 83); k::mul(d, &l[13], &cr[1]); }
    { let (_, d, h) = sp(&mut a.v, 13); k::mul(d, &h[70], &cr[2]); }
    { let (l, d, _) = sp(&mut a.v, 84); k::add(d, &[&l[11], &l[21], &l[22], &l[23], &l[24], &l[25]]); }
    { let (_, d, h) = sp(&mut a.v, 25); k::add(d, &[&h[0], &h[1], &h[2], &h[3], &h[4], &h[5]]); }
    { let (_, d, h) = sp(&mut a.v, 31); k::add(d, &[&h[0], &h[1], &h[2], &h[3], &h[4], &h[5]]); }
    { let (_, d, h) = sp(&mut a.v, 37); k::add(d, &[&h[0], &h[1], &h[2], &h[3], &h[4], &h[5]]); }
    { let (_, d, h) = sp(&mut a.v, 43); k::add(d, &[&h[0], &h[1], &h[2], &h[3], &h[4], &h[5]]); }
    { let (l, d, h) = sp(&mut a.v, 49); k::add(d, &[&h[0], &l[8], &h[1], &h[2], &h[3], &h[4]]); }
    { let (_, d, h) = sp(&mut a.v, 54); k::add(d, &[&h[0], &h[1], &h[2], &h[3], &h[4], &h[5]]); }
    { let (_, d, h) = sp(&mut a.v, 60); k::add(d, &[&h[0], &h[1], &h[2], &h[3], &h[4], &h[5]]); }
    { let (l, d, h) = sp(&mut a.v, 66); k::add(d, &[&h[0], &h[1], &h[2], &h[3], &h[4], &l[12]]); }
    { let (l, d, h) = sp(&mut a.v, 12); k::add(d, &[&h[59], &l[9], &h[60], &h[61], &h[62], &h[63]]); }
    { let (l, d, h) = sp(&mut a.v, 76); k::add(d, &[&h[0], &h[1], &l[20], &h[2], &h[3], &l[10]]); }
    { let (_, d, h) = sp(&mut a.v, 10); k::add(d, &[&h[70], &h[71], &h[8], &h[7], &h[72], &h[2]]); }
    { let (_, d, h) = sp(&mut a.s, 38); k::mul(d, &cc[0], &h[27]); }
    { let (l, d, _) = sp(&mut a.s, 66); k::mul(d, &cc[0], &l[26]); }
    { let (_, d, h) = sp(&mut a.s, 26); k::mul(d, &cc[0], &h[46]); }
    { let (l, d, _) = sp(&mut a.s, 73); k::mul(d, &cc[0], &l[27]); }
    { let (_, d, h) = sp(&mut a.s, 27); k::mul(d, &cc[0], &h[1]); }
    { let (l, d, _) = sp(&mut a.s, 29); k::mul(d, &cc[0], &l[28]); }
    { let (_, d, h) = sp(&mut a.s, 28); k::mul(d, &cc[0], &h[2]); }
    { let (l, d, _) = sp(&mut a.s, 31); k::mul(d, &cc[0], &l[30]); }
    { let (_, d, h) = sp(&mut a.s, 30); k::mul(d, &cc[0], &h[2]); }
    { let (l, d, _) = sp(&mut a.s, 33); k::mul(d, &cc[0], &l[32]); }
    { let (_, d, h) = sp(&mut a.s, 32); k::mul(d, &cc[0], &h[2]); }
    { let (l, d, _) = sp(&mut a.s, 35); k::mul(d, &cc[0], &l[34]); }
    { let (_, d, h) = sp(&mut a.s, 34); k::mul(d, &cc[0], &h[2]); }
    { let (_, d, h) = sp(&mut a.s, 37); k::mul(d, &cc[0], &h[36]); }
    { let (l, d, _) = sp(&mut a.s, 74); k::mul(d, &cc[2], &l[38]); }
    { let (_, d, h) = sp(&mut a.s, 0); k::mul(d, &cc[3], &h[65]); }
    { let (_, d, h) = sp(&mut a.s, 36); k::mul(d, &cc[3], &h[1]); }
    { let (l, d, _) = sp(&mut a.s, 38); k::mul(d, &cc[3], &l[26]); }
    { let (l, d, _) = sp(&mut a.s, 72); k::mul(d, &cc[2], &l[26]); }
    { let (_, d, h) = sp(&mut a.s, 26); k::mul(d, &cc[2], &h[39]); }
    { let (_, d, h) = sp(&mut a.s, 66); k::mul(d, &cc[3], &h[6]); }
    { let (l, d, _) = sp(&mut a.s, 65); k::mul(d, &cc[3], &l[27]); }
    { let (l, d, _) = sp(&mut a.s, 57); k::mul(d, &cc[2], &l[27]); }
    { let (_, d, h) = sp(&mut a.s, 27); k::mul(d, &cc[2], &h[45]); }
    { let (l, d, _) = sp(&mut a.s, 73); k::mul(d, &cc[3], &l[29]); }
    { let (l, d, _) = sp(&mut a.s, 56); k::mul(d, &cc[3], &l[28]); }
    { let (_, d, h) = sp(&mut a.s, 25); k::mul(d, &cc[2], &h[2]); }
    { let (_, d, h) = sp(&mut a.s, 28); k::mul(d, &cc[2], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 29); k::mul(d, &cc[3], &h[1]); }
    { let (l, d, _) = sp(&mut a.s, 47); k::mul(d, &cc[3], &l[30]); }
    { let (l, d, _) = sp(&mut a.s, 46); k::mul(d, &cc[2], &l[30]); }
    { let (_, d, h) = sp(&mut a.s, 30); k::mul(d, &cc[2], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 31); k::mul(d, &cc[3], &h[1]); }
    { let (l, d, _) = sp(&mut a.s, 68); k::mul(d, &cc[3], &l[32]); }
    { let (l, d, _) = sp(&mut a.s, 67); k::mul(d, &cc[2], &l[32]); }
    { let (_, d, h) = sp(&mut a.s, 32); k::mul(d, &cc[2], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 33); k::mul(d, &cc[2], &h[1]); }
    { let (_, d, h) = sp(&mut a.s, 22); k::mul(d, &cc[3], &h[11]); }
    { let (l, d, _) = sp(&mut a.s, 53); k::mul(d, &cc[3], &l[35]); }
    { let (_, d, h) = sp(&mut a.s, 35); k::mul(d, &cc[3], &h[1]); }
    { let (l, d, _) = sp(&mut a.s, 52); k::mul(d, &cc[2], &l[37]); }
    { let (l, d, _) = sp(&mut a.s, 37); k::mul(d, &cc[2], &l[34]); }
    { let (_, d, h) = sp(&mut a.v, 13); k::mul(d, &h[70], &cc[1]); }
    { let (l, d, _) = sp(&mut a.v, 84); k::mul(d, &l[25], &cc[1]); }
    { let (_, d, h) = sp(&mut a.v, 25); k::mul(d, &h[5], &cc[1]); }
    { let (_, d, h) = sp(&mut a.v, 31); k::mul(d, &h[5], &cc[1]); }
    { let (_, d, h) = sp(&mut a.v, 37); k::mul(d, &h[5], &cc[1]); }
    { let (_, d, h) = sp(&mut a.v, 43); k::mul(d, &h[5], &cc[1]); }
    { let (_, d, h) = sp(&mut a.v, 49); k::mul(d, &h[4], &cc[1]); }
    { let (_, d, h) = sp(&mut a.v, 54); k::mul(d, &h[5], &cc[1]); }
    { let (_, d, h) = sp(&mut a.v, 60); k::mul(d, &h[5], &cc[1]); }
    { let (l, d, _) = sp(&mut a.v, 66); k::mul(d, &l[12], &cc[1]); }
    { let (_, d, h) = sp(&mut a.v, 12); k::mul(d, &h[63], &cc[1]); }
    { let (l, d, _) = sp(&mut a.v, 76); k::mul(d, &l[10], &cc[1]); }
    { let (_, d, h) = sp(&mut a.v, 10); k::propagate_vector(d, &h[2], &mm[11], &cr[0], &cr[5]); }
    { let (_, d, h) = sp(&mut a.v, 13); k::propagate_vector(d, &h[70], &mm[6], &cr[0], &cr[6]); }
    { let (l, d, _) = sp(&mut a.v, 84); k::propagate_vector(d, &l[25], &mm[9], &cr[0], &cr[6]); }
    { let (_, d, h) = sp(&mut a.v, 25); k::propagate_vector(d, &h[5], &mm[11], &cr[0], &cr[5]); }
    { let (_, d, h) = sp(&mut a.v, 31); k::propagate_vector(d, &h[5], &mm[6], &cr[0], &cr[6]); }
    { let (_, d, h) = sp(&mut a.v, 37); k::propagate_vector(d, &h[5], &mm[9], &cr[0], &cr[6]); }
    { let (_, d, h) = sp(&mut a.v, 43); k::propagate_vector(d, &h[5], &mm[11], &cr[0], &cr[5]); }
    { let (_, d, h) = sp(&mut a.v, 49); k::propagate_vector(d, &h[4], &mm[6], &cr[0], &cr[6]); }
    { let (_, d, h) = sp(&mut a.v, 54); k::propagate_vector(d, &h[5], &mm[9], &cr[0], &cr[6]); }
    { let (_, d, h) = sp(&mut a.v, 60); k::propagate_vector(d, &h[5], &mm[9], &cr[0], &cr[6]); }
    { let (l, d, _) = sp(&mut a.v, 66); k::propagate_vector(d, &l[12], &mm[6], &cr[0], &cr[6]); }
    { let (_, d, h) = sp(&mut a.v, 12); k::propagate_vector(d, &h[63], &mm[11], &cr[0], &cr[5]); }
    k::metric(&mut a.s[34], &a.v[16], &a.v[10]);
    k::metric(&mut a.s[62], &a.v[17], &a.v[10]);
    k::metric(&mut a.s[61], &a.v[0], &a.v[10]);
    k::metric(&mut a.s[20], &a.v[1], &a.v[10]);
    k::metric(&mut a.s[64], &a.v[16], &a.v[13]);
    k::metric(&mut a.s[63], &a.v[15], &a.v[13]);
    k::metric(&mut a.s[51], &a.v[0], &a.v[13]);
    k::metric(&mut a.s[50], &a.v[3], &a.v[13]);
    k::metric(&mut a.s[16], &a.v[16], &a.v[84]);
    k::metric(&mut a.s[59], &a.v[14], &a.v[84]);
    k::metric(&mut a.s[58], &a.v[0], &a.v[84]);
    k::metric(&mut a.s[14], &a.v[2], &a.v[84]);
    k::metric(&mut a.s[41], &a.v[0], &a.v[25]);
    k::metric(&mut a.s[40], &a.v[4], &a.v[25]);
    k::metric(&mut a.s[17], &a.v[16], &a.v[31]);
    k::metric(&mut a.s[12], &a.v[15], &a.v[31]);
    k::metric(&mut a.s[45], &a.v[0], &a.v[31]);
    k::metric(&mut a.s[44], &a.v[5], &a.v[31]);
    k::metric(&mut a.s[11], &a.v[16], &a.v[37]);
    k::metric(&mut a.s[43], &a.v[14], &a.v[37]);
    k::metric(&mut a.s[42], &a.v[0], &a.v[37]);
    k::metric(&mut a.s[13], &a.v[2], &a.v[37]);
    k::metric(&mut a.s[7], &a.v[0], &a.v[43]);
    k::metric(&mut a.s[5], &a.v[4], &a.v[43]);
    k::metric(&mut a.s[9], &a.v[16], &a.v[49]);
    k::metric(&mut a.s[39], &a.v[15], &a.v[49]);
    k::metric(&mut a.s[75], &a.v[0], &a.v[49]);
    k::metric(&mut a.s[76], &a.v[3], &a.v[49]);
    k::metric(&mut a.s[77], &a.v[16], &a.v[54]);
    k::metric(&mut a.s[78], &a.v[14], &a.v[54]);
    k::metric(&mut a.s[79], &a.v[0], &a.v[54]);
    k::metric(&mut a.s[80], &a.v[6], &a.v[54]);
    k::metric(&mut a.s[81], &a.v[7], &a.v[25]);
    k::metric(&mut a.s[82], &a.v[1], &a.v[25]);
    k::metric(&mut a.s[83], &a.v[7], &a.v[13]);
    k::metric(&mut a.s[84], &a.v[5], &a.v[13]);
    k::metric(&mut a.s[85], &a.v[16], &a.v[60]);
    k::metric(&mut a.s[86], &a.v[14], &a.v[60]);
    k::metric(&mut a.s[87], &a.v[7], &a.v[60]);
    k::metric(&mut a.s[88], &a.v[2], &a.v[60]);
    k::metric(&mut a.s[89], &a.v[7], &a.v[43]);
    k::metric(&mut a.s[90], &a.v[1], &a.v[43]);
    k::metric(&mut a.s[91], &a.v[16], &a.v[66]);
    k::metric(&mut a.s[92], &a.v[15], &a.v[66]);
    k::metric(&mut a.s[93], &a.v[7], &a.v[66]);
    k::metric(&mut a.s[94], &a.v[3], &a.v[66]);
    k::metric(&mut a.s[95], &a.v[7], &a.v[84]);
    k::metric(&mut a.s[96], &a.v[6], &a.v[84]);
    k::metric(&mut a.s[97], &a.v[16], &a.v[12]);
    k::metric(&mut a.s[98], &a.v[17], &a.v[12]);
    k::metric(&mut a.s[99], &a.v[7], &a.v[12]);
    k::metric(&mut a.s[100], &a.v[4], &a.v[12]);
    k::metric(&mut a.s[101], &a.v[7], &a.v[49]);
    k::metric(&mut a.s[102], &a.v[5], &a.v[49]);
    k::metric(&mut a.s[103], &a.v[7], &a.v[37]);
    k::metric(&mut a.s[104], &a.v[6], &a.v[37]);
    k::pmom(&mut a.v[37], &mm[11]);
    k::pmom(&mut a.v[49], &mm[6]);
    k::pmom(&mut a.v[12], &mm[9]);
    { let (l, d, _) = sp(&mut a.s, 105); k::mul(d, &l[34], &l[8]); }
    { let (l, d, h) = sp(&mut a.s, 34); k::mul(d, &h[27], &l[8]); }
    { let (l, d, h) = sp(&mut a.s, 62); k::mul(d, &h[1], &l[4]); }
    { let (l, d, h) = sp(&mut a.s, 8); k::mul(d, &h[54], &l[4]); }
    { let (l, d, _) = sp(&mut a.s, 106); k::mul(d, &l[48], &l[51]); }
    { let (l, d, _) = sp(&mut a.s, 107); k::mul(d, &l[49], &l[50]); }
    { let (l, d, _) = sp(&mut a.s, 108); k::mul(d, &l[16], &l[6]); }
    { let (l, d, _) = sp(&mut a.s, 109); k::mul(d, &l[59], &l[6]); }
    { let (l, d, _) = sp(&mut a.s, 110); k::mul(d, &l[54], &l[58]); }
    { let (l, d, _) = sp(&mut a.s, 111); k::mul(d, &l[55], &l[14]); }
    { let (l, d, _) = sp(&mut a.s, 112); k::mul(d, &l[17], &l[10]); }
    { let (l, d, _) = sp(&mut a.s, 17); k::mul(d, &l[12], &l[10]); }
    { let (_, d, h) = sp(&mut a.s, 12); k::mul(d, &h[47], &h[32]); }
    { let (_, d, h) = sp(&mut a.s, 10); k::mul(d, &h[38], &h[33]); }
    { let (l, d, _) = sp(&mut a.s, 113); k::mul(d, &l[11], &l[6]); }
    { let (l, d, _) = sp(&mut a.s, 114); k::mul(d, &l[43], &l[6]); }
    { let (_, d, h) = sp(&mut a.s, 6); k::mul(d, &h[47], &h[35]); }
    { let (l, d, _) = sp(&mut a.s, 115); k::mul(d, &l[55], &l[13]); }
    { let (l, d, _) = sp(&mut a.s, 116); k::mul(d, &l[9], &l[4]); }
    { let (l, d, _) = sp(&mut a.s, 117); k::mul(d, &l[39], &l[4]); }
    { let (_, d, h) = sp(&mut a.s, 4); k::mul(d, &h[43], &h[70]); }
    { let (l, d, _) = sp(&mut a.s, 118); k::mul(d, &l[49], &l[76]); }
    { let (l, d, h) = sp(&mut a.s, 49); k::mul(d, &h[27], &l[15]); }
    { let (l, d, h) = sp(&mut a.s, 77); k::mul(d, &h[0], &l[15]); }
    { let (l, d, h) = sp(&mut a.s, 78); k::mul(d, &l[69], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 15); k::mul(d, &h[39], &h[64]); }
    { let (l, d, h) = sp(&mut a.s, 55); k::mul(d, &h[8], &l[18]); }
    { let (l, d, _) = sp(&mut a.s, 64); k::mul(d, &l[63], &l[18]); }
    { let (l, d, h) = sp(&mut a.s, 63); k::mul(d, &l[60], &h[19]); }
    { let (l, d, _) = sp(&mut a.s, 119); k::mul(d, &l[70], &l[84]); }
    { let (l, d, _) = sp(&mut a.s, 120); k::mul(d, &l[85], &l[19]); }
    { let (l, d, h) = sp(&mut a.s, 85); k::mul(d, &h[0], &l[19]); }
    { let (l, d, h) = sp(&mut a.s, 86); k::mul(d, &l[54], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 54); k::mul(d, &h[16], &h[33]); }
    { let (_, d, h) = sp(&mut a.s, 19); k::mul(d, &h[71], &h[1]); }
    { let (l, d, h) = sp(&mut a.s, 91); k::mul(d, &h[0], &l[21]); }
    { let (l, d, h) = sp(&mut a.s, 92); k::mul(d, &l[48], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 48); k::mul(d, &h[21], &h[45]); }
    { let (l, d, h) = sp(&mut a.s, 21); k::mul(d, &l[16], &h[1]); }
    { let (_, d, h) = sp(&mut a.s, 16); k::mul(d, &h[42], &h[6]); }
    { let (_, d, h) = sp(&mut a.s, 59); k::mul(d, &h[9], &h[35]); }
    { let (l, d, _) = sp(&mut a.s, 121); k::mul(d, &l[71], &l[96]); }
    { let (l, d, _) = sp(&mut a.s, 122); k::mul(d, &l[97], &l[24]); }
    { let (l, d, h) = sp(&mut a.s, 97); k::mul(d, &h[0], &l[24]); }
    { let (l, d, _) = sp(&mut a.s, 98); k::mul(d, &l[9], &l[18]); }
    { let (_, d, h) = sp(&mut a.s, 9); k::mul(d, &h[29], &h[8]); }
    { let (_, d, h) = sp(&mut a.s, 18); k::mul(d, &h[41], &h[82]); }
    { let (_, d, h) = sp(&mut a.s, 60); k::mul(d, &h[9], &h[41]); }
    { let (l, d, _) = sp(&mut a.s, 70); k::mul(d, &l[11], &l[23]); }
    { let (_, d, h) = sp(&mut a.s, 11); k::mul(d, &h[31], &h[11]); }
    { let (_, d, h) = sp(&mut a.s, 23); k::mul(d, &h[45], &h[79]); }
    { let (_, d, h) = sp(&mut a.s, 69); k::mul(d, &h[1], &h[34]); }
    k::metric(&mut a.s[71], &a.v[37], &a.v[1]);
    k::metric(&mut a.s[43], &a.v[37], &a.v[0]);
    k::metric(&mut a.s[39], &a.v[49], &a.v[3]);
    k::metric(&mut a.s[24], &a.v[49], &a.v[0]);
    k::metric(&mut a.s[123], &a.v[12], &a.v[2]);
    k::metric(&mut a.s[124], &a.v[12], &a.v[0]);
    k::metric(&mut a.s[125], &a.v[37], &a.v[4]);
    k::metric(&mut a.s[126], &a.v[49], &a.v[5]);
    k::metric(&mut a.s[127], &a.v[12], &a.v[6]);
    k::metric(&mut a.s[128], &a.v[37], &a.v[7]);
    k::metric(&mut a.s[129], &a.v[49], &a.v[7]);
    k::metric(&mut a.s[130], &a.v[12], &a.v[7]);
    { let (l, d, _) = sp(&mut a.s, 131); k::mul(d, &l[71], &l[61]); }
    { let (l, d, _) = sp(&mut a.s, 61); k::mul(d, &l[43], &l[20]); }
    { let (_, d, h) = sp(&mut a.s, 20); k::mul(d, &h[18], &h[30]); }
    { let (l, d, _) = sp(&mut a.s, 51); k::mul(d, &l[24], &l[50]); }
    { let (_, d, h) = sp(&mut a.s, 50); k::mul(d, &h[72], &h[7]); }
    { let (l, d, h) = sp(&mut a.s, 58); k::mul(d, &h[65], &l[14]); }
    { let (_, d, h) = sp(&mut a.s, 14); k::mul(d, &h[110], &h[26]); }
    { let (l, d, h) = sp(&mut a.s, 41); k::mul(d, &h[1], &l[40]); }
    { let (_, d, h) = sp(&mut a.s, 40); k::mul(d, &h[85], &h[4]); }
    { let (l, d, _) = sp(&mut a.s, 45); k::mul(d, &l[24], &l[44]); }
    { let (l, d, h) = sp(&mut a.s, 44); k::mul(d, &h[78], &l[42]); }
    { let (l, d, h) = sp(&mut a.s, 42); k::mul(d, &h[81], &l[13]); }
    { let (l, d, h) = sp(&mut a.s, 13); k::mul(d, &h[111], &l[7]); }
    { let (l, d, h) = sp(&mut a.s, 7); k::mul(d, &h[35], &l[5]); }
    { let (_, d, h) = sp(&mut a.s, 5); k::mul(d, &h[33], &h[69]); }
    { let (l, d, h) = sp(&mut a.s, 75); k::mul(d, &l[24], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 76); k::mul(d, &h[50], &h[2]); }
    { let (_, d, h) = sp(&mut a.s, 79); k::mul(d, &h[44], &h[0]); }
    { let (l, d, h) = sp(&mut a.s, 80); k::mul(d, &l[71], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 81); k::mul(d, &h[46], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 82); k::mul(d, &h[43], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 83); k::mul(d, &h[45], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 84); k::mul(d, &h[38], &h[2]); }
    { let (_, d, h) = sp(&mut a.s, 87); k::mul(d, &h[42], &h[0]); }
    { let (l, d, h) = sp(&mut a.s, 88); k::mul(d, &l[71], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 89); k::mul(d, &h[38], &h[0]); }
    { let (l, d, h) = sp(&mut a.s, 90); k::mul(d, &l[39], &h[2]); }
    { let (_, d, h) = sp(&mut a.s, 93); k::mul(d, &h[35], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 94); k::mul(d, &h[32], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 95); k::mul(d, &h[34], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 96); k::mul(d, &h[28], &h[2]); }
    { let (_, d, h) = sp(&mut a.s, 99); k::mul(d, &h[28], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 100); k::mul(d, &h[25], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 101); k::mul(d, &h[27], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 102); k::mul(d, &h[24], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 103); k::mul(d, &h[26], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 104); k::mul(d, &h[0], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 105); k::mul(d, &l[34], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 34); k::mul(d, &h[27], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 62); k::mul(d, &l[8], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 8); k::mul(d, &h[97], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 106); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 107); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 108); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 109); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 110); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 111); k::mul(d, &h[0], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 112); k::mul(d, &l[17], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 17); k::mul(d, &l[12], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 12); k::mul(d, &l[10], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 10); k::mul(d, &h[102], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 113); k::mul(d, &h[0], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 114); k::mul(d, &l[6], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 6); k::mul(d, &h[108], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 115); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 116); k::mul(d, &h[0], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 117); k::mul(d, &l[4], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 4); k::mul(d, &h[113], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 118); k::mul(d, &l[49], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 49); k::mul(d, &h[27], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 77); k::mul(d, &h[0], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 78); k::mul(d, &l[15], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 15); k::mul(d, &h[39], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 55); k::mul(d, &h[8], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 64); k::mul(d, &l[63], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 63); k::mul(d, &h[55], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 119); k::mul(d, &h[0], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 120); k::mul(d, &l[85], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 85); k::mul(d, &h[0], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 86); k::mul(d, &l[54], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 54); k::mul(d, &l[19], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 19); k::mul(d, &h[71], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 91); k::mul(d, &h[0], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 92); k::mul(d, &l[48], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 48); k::mul(d, &l[21], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 21); k::mul(d, &l[16], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 16); k::mul(d, &h[42], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 59); k::mul(d, &h[61], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 121); k::mul(d, &h[0], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 122); k::mul(d, &l[97], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 97); k::mul(d, &h[0], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 98); k::mul(d, &l[9], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 9); k::mul(d, &h[8], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 18); k::mul(d, &h[41], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 60); k::mul(d, &h[9], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 70); k::mul(d, &l[11], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 11); k::mul(d, &h[11], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 23); k::mul(d, &h[45], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 69); k::mul(d, &h[61], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 131); k::mul(d, &l[61], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 61); k::mul(d, &l[20], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 20); k::mul(d, &h[30], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 51); k::mul(d, &l[50], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 50); k::mul(d, &h[7], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 58); k::mul(d, &l[14], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 14); k::mul(d, &h[26], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 41); k::mul(d, &l[40], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 40); k::mul(d, &h[4], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 45); k::mul(d, &l[44], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 44); k::mul(d, &l[42], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 42); k::mul(d, &l[13], &cr[1]); }
    { let (l, d, _) = sp(&mut a.s, 13); k::mul(d, &l[7], &cr[2]); }
    { let (l, d, _) = sp(&mut a.s, 7); k::mul(d, &l[5], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 5); k::mul(d, &h[69], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 75); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 76); k::mul(d, &h[2], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 79); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 80); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 81); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 82); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 83); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 84); k::mul(d, &h[2], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 87); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 88); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 89); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 90); k::mul(d, &h[2], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 93); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 94); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 95); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 96); k::mul(d, &h[2], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 99); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 100); k::mul(d, &h[0], &cr[2]); }
    { let (_, d, h) = sp(&mut a.s, 101); k::mul(d, &h[0], &cr[1]); }
    { let (_, d, h) = sp(&mut a.s, 102); k::mul(d, &h[0], &cr[2]); }
    { let (l, d, h) = sp(&mut a.s, 103); k::add(d, &[&h[0], &h[1], &l[69], &h[27]]); }
    { let (l, d, _) = sp(&mut a.s, 131); k::add(d, &[&l[34], &l[62], &l[8], &l[61], &l[106], &l[20]]); }
    { let (_, d, h) = sp(&mut a.s, 20); k::add(d, &[&h[86], &h[87], &h[88], &h[30], &h[89], &h[29]]); }
    { let (l, d, h) = sp(&mut a.s, 50); k::add(d, &[&h[7], &l[14]]); }
    { let (l, d, h) = sp(&mut a.s, 14); k::add(d, &[&h[96], &h[97], &h[2], &h[26], &l[12], &h[25]]); }
    { let (l, d, h) = sp(&mut a.s, 40); k::add(d, &[&l[10], &h[72], &h[73], &h[4], &l[6], &h[3]]); }
    { let (l, d, _) = sp(&mut a.s, 44); k::add(d, &[&l[42], &l[13]]); }
    { let (l, d, h) = sp(&mut a.s, 13); k::add(d, &[&h[101], &h[102], &h[103], &l[7], &l[4], &l[5]]); }
    { let (_, d, h) = sp(&mut a.s, 5); k::add(d, &[&h[112], &h[43], &h[71], &h[69], &h[72], &h[70]]); }
    { let (_, d, h) = sp(&mut a.s, 76); k::add(d, &[&h[2], &h[3]]); }
    { let (l, d, h) = sp(&mut a.s, 80); k::add(d, &[&l[15], &l[55], &l[64], &h[0], &l[63], &h[1]]); }
    { let (_, d, h) = sp(&mut a.s, 82); k::add(d, &[&h[36], &h[37], &h[2], &h[0], &h[3], &h[1]]); }
    { let (_, d, h) = sp(&mut a.s, 84); k::add(d, &[&h[2], &h[3]]); }
    { let (l, d, h) = sp(&mut a.s, 88); k::add(d, &[&l[54], &l[19], &h[2], &h[0], &h[3], &h[1]]); }
    { let (l, d, h) = sp(&mut a.s, 90); k::add(d, &[&l[48], &l[21], &l[16], &h[2], &l[59], &h[3]]); }
    { let (_, d, h) = sp(&mut a.s, 94); k::add(d, &[&h[26], &h[27], &h[0], &h[1]]); }
    { let (l, d, h) = sp(&mut a.s, 96); k::add(d, &[&h[0], &h[1], &l[9], &h[2], &l[18], &h[3]]); }
    { let (l, d, h) = sp(&mut a.s, 100); k::add(d, &[&l[60], &l[70], &l[11], &h[0], &l[23], &h[1]]); }
    { let (_, d, h) = sp(&mut a.s, 102); k::mul(d, &cc[1], &h[0]); }
    { let (_, d, h) = sp(&mut a.s, 103); k::mul(d, &cc[1], &h[27]); }
    { let (l, d, _) = sp(&mut a.s, 131); k::mul(d, &cc[1], &l[20]); }
    { let (_, d, h) = sp(&mut a.s, 20); k::mul(d, &cc[1], &h[29]); }
    { let (l, d, _) = sp(&mut a.s, 50); k::mul(d, &cc[1], &l[14]); }
    { let (_, d, h) = sp(&mut a.s, 14); k::mul(d, &cc[1], &h[25]); }
    { let (_, d, h) = sp(&mut a.s, 40); k::mul(d, &cc[1], &h[3]); }
    { let (l, d, _) = sp(&mut a.s, 44); k::mul(d, &cc[1], &l[13]); }
    { let (l, d, _) = sp(&mut a.s, 13); k::mul(d, &cc[1], &l[5]); }
    { let (_, d, h) = sp(&mut a.s, 5); k::mul(d, &cc[1], &h[70]); }
    { let (_, d, h) = sp(&mut a.s, 76); k::mul(d, &cc[1], &h[3]); }
    { let (_, d, h) = sp(&mut a.s, 80); k::mul(d, &cc[1], &h[1]); }
    { let (_, d, h) = sp(&mut a.s, 82); k::mul(d, &cc[1], &h[1]); }
    { let (_, d, h) = sp(&mut a.s, 84); k::mul(d, &cc[1], &h[3]); }
    { let (_, d, h) = sp(&mut a.s, 88); k::mul(d, &cc[1], &h[1]); }
    { let (_, d, h) = sp(&mut a.s, 90); k::mul(d, &cc[1], &h[3]); }
    { let (_, d, h) = sp(&mut a.s, 94); k::mul(d, &cc[1], &h[1]); }
    { let (_, d, h) = sp(&mut a.s, 96); k::mul(d, &cc[1], &h[3]); }
    { let (_, d, h) = sp(&mut a.s, 100); k::mul(d, &cc[2], &h[1]); }
    { let (_, d, h) = sp(&mut a.s, 3); k::mul(d, &cc[3], &h[99]); }
    { let (_, d, h) = sp(&mut a.s, 23); k::mul(d, &cc[3], &h[78]); }
    { let (_, d, h) = sp(&mut a.s, 101); k::mul(d, &cc[3], &h[29]); }
    { let (_, d, h) = sp(&mut a.s, 11); k::mul(d, &cc[2], &h[119]); }
    { let (_, d, h) = sp(&mut a.s, 70); k::mul(d, &cc[2], &h[32]); }
    { let (l, d, _) = sp(&mut a.s, 60); k::mul(d, &cc[3], &l[50]); }
    { let (l, d, _) = sp(&mut a.s, 18); k::mul(d, &cc[3], &l[14]); }
    { let (l, d, _) = sp(&mut a.s, 99); k::mul(d, &cc[2], &l[14]); }
    { let (_, d, h) = sp(&mut a.s, 9); k::mul(d, &cc[2], &h[40]); }
    { let (l, d, _) = sp(&mut a.s, 98); k::mul(d, &cc[3], &l[44]); }
    { let (l, d, _) = sp(&mut a.s, 97); k::mul(d, &cc[3], &l[13]); }
    { let (l, d, _) = sp(&mut a.s, 95); k::mul(d, &cc[2], &l[13]); }
    { let (l, d, _) = sp(&mut a.s, 122); k::mul(d, &cc[2], &l[44]); }
    { let (l, d, _) = sp(&mut a.s, 121); k::mul(d, &cc[3], &l[76]); }
    { let (_, d, h) = sp(&mut a.s, 59); k::mul(d, &cc[3], &h[20]); }
    { let (l, d, _) = sp(&mut a.s, 93); k::mul(d, &cc[2], &l[80]); }
    { let (_, d, h) = sp(&mut a.s, 16); k::mul(d, &cc[2], &h[59]); }
    { let (_, d, h) = sp(&mut a.s, 21); k::mul(d, &cc[3], &h[62]); }
    { let (_, d, h) = sp(&mut a.s, 48); k::mul(d, &cc[3], &h[39]); }
    { let (l, d, _) = sp(&mut a.s, 92); k::mul(d, &cc[2], &l[88]); }
    { let (l, d, _) = sp(&mut a.s, 89); k::mul(d, &cc[2], &l[84]); }
    { let (l, d, _) = sp(&mut a.s, 91); k::mul(d, &cc[2], &l[90]); }
    { let (_, d, h) = sp(&mut a.s, 19); k::mul(d, &cc[3], &h[74]); }
    { let (_, d, h) = sp(&mut a.s, 54); k::mul(d, &cc[3], &h[35]); }
    { let (_, d, h) = sp(&mut a.s, 87); k::mul(d, &cc[3], &h[8]); }
    { let (_, d, h) = sp(&mut a.s, 2); k::mul(d, &cc[2], &h[93]); }
    { let (_, d, h) = sp(&mut a.s, 86); k::mul(d, &cc[2], &h[7]); }
    { let (l, d, h) = sp(&mut a.s, 1); k::add(d, &[&h[72], &l[0], &h[98], &h[1]]); }
    { let (_, d, h) = sp(&mut a.s, 3); k::add(d, &[&h[32], &h[34], &h[19], &h[97]]); }
    { let (l, d, _) = sp(&mut a.s, 101); k::add(d, &[&l[72], &l[26], &l[11], &l[70]]); }
    { let (l, d, _) = sp(&mut a.s, 70); k::add(d, &[&l[66], &l[60]]); }
    { let (l, d, h) = sp(&mut a.s, 60); k::add(d, &[&h[4], &l[18]]); }
    { let (l, d, h) = sp(&mut a.s, 18); k::add(d, &[&h[38], &h[8], &h[80], &l[9]]); }
    { let (_, d, h) = sp(&mut a.s, 9); k::add(d, &[&h[63], &h[88]]); }
    { let (l, d, _) = sp(&mut a.s, 98); k::add(d, &[&l[56], &l[97]]); }
    { let (l, d, h) = sp(&mut a.s, 97); k::add(d, &[&l[25], &l[28], &l[95], &h[24]]); }
    { let (l, d, _) = sp(&mut a.s, 122); k::add(d, &[&l[29], &l[121]]); }
    { let (l, d, _) = sp(&mut a.s, 121); k::add(d, &[&l[47], &l[59]]); }
    { let (l, d, h) = sp(&mut a.s, 59); k::add(d, &[&l[46], &l[30], &h[33], &l[16]]); }
    { let (_, d, h) = sp(&mut a.s, 16); k::add(d, &[&h[14], &h[4]]); }
    { let (_, d, h) = sp(&mut a.s, 21); k::add(d, &[&h[46], &h[26]]); }
    { let (l, d, h) = sp(&mut a.s, 48); k::add(d, &[&h[18], &l[32], &h[43], &h[40]]); }
    { let (l, d, h) = sp(&mut a.s, 89); k::add(d, &[&l[33], &l[22], &h[1], &l[19]]); }
    { let (_, d, h) = sp(&mut a.s, 19); k::add(d, &[&h[33], &h[15], &h[34], &h[67]]); }
    { let (l, d, _) = sp(&mut a.s, 87); k::add(d, &[&l[52], &l[37], &l[2], &l[86]]); }
}
