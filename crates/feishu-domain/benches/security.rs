//! Benchmarks for SecurityGuard (is_dm_allowed, is_group_allowed).

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn quick_config() -> Criterion {
    Criterion::default()
        .sample_size(20)
        .warm_up_time(std::time::Duration::from_millis(100))
        .measurement_time(std::time::Duration::from_millis(500))
}
use feishu_domain::security::{DmPolicy, GroupPolicy, SecurityGuard};

fn bench_is_dm_allowed_open(c: &mut Criterion) {
    let guard = SecurityGuard::with_policies(
        vec!["*".into()],
        Some(DmPolicy::Open),
        vec![],
        None,
    );
    c.bench_function("security_is_dm_allowed_open", |b| {
        b.iter(|| guard.is_dm_allowed(black_box("ou_anyuser")))
    });
}

fn bench_is_dm_allowed_pairing_hit(c: &mut Criterion) {
    let guard = SecurityGuard::new(vec![
        "ou_user1".into(),
        "ou_user2".into(),
        "ou_user3".into(),
    ]);
    c.bench_function("security_is_dm_allowed_pairing_hit", |b| {
        b.iter(|| guard.is_dm_allowed(black_box("ou_user2")))
    });
}

fn bench_is_dm_allowed_pairing_miss(c: &mut Criterion) {
    let guard = SecurityGuard::new(vec![
        "ou_user1".into(),
        "ou_user2".into(),
        "ou_user3".into(),
    ]);
    c.bench_function("security_is_dm_allowed_pairing_miss", |b| {
        b.iter(|| guard.is_dm_allowed(black_box("ou_unknown")))
    });
}

fn bench_is_group_allowed_open(c: &mut Criterion) {
    let guard = SecurityGuard::with_policies(
        vec![],
        Some(DmPolicy::Deny),
        vec![],
        Some(GroupPolicy::Open),
    );
    c.bench_function("security_is_group_allowed_open", |b| {
        b.iter(|| guard.is_group_allowed(black_box("ou_anyuser")))
    });
}

fn bench_is_group_allowed_allowlist_hit(c: &mut Criterion) {
    let guard = SecurityGuard::with_policies(
        vec![],
        Some(DmPolicy::Deny),
        vec!["ou_grp1".into(), "ou_grp2".into(), "ou_grp3".into()],
        Some(GroupPolicy::Allowlist),
    );
    c.bench_function("security_is_group_allowed_allowlist_hit", |b| {
        b.iter(|| guard.is_group_allowed(black_box("ou_grp2")))
    });
}

criterion_group! {
    name = benches;
    config = quick_config();
    targets =
        bench_is_dm_allowed_open,
        bench_is_dm_allowed_pairing_hit,
        bench_is_dm_allowed_pairing_miss,
        bench_is_group_allowed_open,
        bench_is_group_allowed_allowlist_hit,
}
criterion_main!(benches);
