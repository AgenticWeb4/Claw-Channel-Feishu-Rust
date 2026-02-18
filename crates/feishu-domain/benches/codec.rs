//! Benchmarks for message codec (decode, encode, strip_bot_mentions).

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn quick_config() -> Criterion {
    Criterion::default()
        .sample_size(20)
        .warm_up_time(std::time::Duration::from_millis(100))
        .measurement_time(std::time::Duration::from_millis(500))
}
use feishu_domain::codec::{
    decode_message_content, encode_text_message, strip_bot_mentions,
};
use feishu_domain::model::{FeishuMention, FeishuSenderId};

fn bench_decode_text(c: &mut Criterion) {
    let content = r#"{"text":"Hello world from Feishu"}"#;
    c.bench_function("codec_decode_text", |b| {
        b.iter(|| decode_message_content(black_box(content), black_box("text")))
    });
}

fn bench_decode_unknown_type(c: &mut Criterion) {
    let content = r#"{"some":"data"}"#;
    c.bench_function("codec_decode_unknown_type", |b| {
        b.iter(|| decode_message_content(black_box(content), black_box("unknown")))
    });
}

fn bench_encode_text(c: &mut Criterion) {
    let text = "Hello world";
    c.bench_function("codec_encode_text", |b| {
        b.iter(|| encode_text_message(black_box(text)))
    });
}

fn bench_strip_bot_mentions(c: &mut Criterion) {
    let mentions = vec![
        FeishuMention {
            key: "@_user_1".into(),
            id: FeishuSenderId {
                open_id: Some("ou_bot".into()),
                user_id: None,
                union_id: None,
            },
            name: "MyBot".into(),
        },
        FeishuMention {
            key: "@_user_2".into(),
            id: FeishuSenderId {
                open_id: Some("ou_user2".into()),
                user_id: None,
                union_id: None,
            },
            name: "User2".into(),
        },
    ];
    let text = "@_user_1 Hello @_user_2 please help";
    c.bench_function("codec_strip_bot_mentions", |b| {
        b.iter(|| strip_bot_mentions(black_box(text), black_box(&mentions)))
    });
}

criterion_group! {
    name = benches;
    config = quick_config();
    targets = bench_decode_text, bench_decode_unknown_type, bench_encode_text, bench_strip_bot_mentions
}
criterion_main!(benches);
