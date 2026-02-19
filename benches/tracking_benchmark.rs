use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tracking_numbers::track;

const TEST_NUMBERS: &[(&str, &str, &str)] = &[
    ("UPS", "1Z5R89390357567127", "1Z5R89390357567120"),
    ("FedEx", "790535312345", "790535312340"),
    ("USPS", "9400111202555842332908", "9400111202555842332900"),
    ("DHL", "3318810025", "3318810011"),
    ("CanadaPost", "0073938000549297", "0073938000549292"),
    ("DPD", "09998000020033F", "09998000020033A"),
    ("Amazon", "TBA123456789000", "TBA123456789999"),
    ("LaserShip", "1LS12345678901234", "1LS12345678901230"),
    ("OnTrac", "C12345678901234", "C12345678901230"),
    ("Landmark", "LM12345678", "LM12345670"),
    ("OldDominion", "12345678", "12345670"),
    ("S10", "RB123456785GB", "RB123456785XX"),
];

fn bench_valid_numbers(c: &mut Criterion) {
    let mut group = c.benchmark_group("valid_tracking_numbers");

    for (courier, valid_num, _) in TEST_NUMBERS.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(courier),
            valid_num,
            |b, num| {
                b.iter(|| {
                    track(black_box(num))
                });
            },
        );
    }

    group.finish();
}

fn bench_invalid_numbers(c: &mut Criterion) {
    let mut group = c.benchmark_group("invalid_tracking_numbers");

    for (courier, _, invalid_num) in TEST_NUMBERS.iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(courier),
            invalid_num,
            |b, num| {
                b.iter(|| {
                    track(black_box(num))
                });
            },
        );
    }

    group.finish();
}

fn bench_mixed_workload(c: &mut Criterion) {
    c.bench_function("mixed_workload", |b| {
        b.iter(|| {
            for (_, valid_num, invalid_num) in TEST_NUMBERS.iter() {
                track(black_box(valid_num));
                track(black_box(invalid_num));
            }
        });
    });
}

fn bench_single_track_ups(c: &mut Criterion) {
    c.bench_function("single_track_ups", |b| {
        b.iter(|| {
            track(black_box("1Z5R89390357567127"))
        });
    });
}

fn bench_sequential_ups(c: &mut Criterion) {
    c.bench_function("sequential_10_ups", |b| {
        b.iter(|| {
            for _ in 0..10 {
                track(black_box("1Z5R89390357567127"));
            }
        });
    });
}

fn bench_not_found(c: &mut Criterion) {
    c.bench_function("not_found", |b| {
        b.iter(|| {
            track(black_box("INVALID123456789"))
        });
    });
}

fn bench_first_match(c: &mut Criterion) {
    // Test a number that matches the first courier loaded
    c.bench_function("first_courier_match", |b| {
        b.iter(|| {
            track(black_box("TBA123456789000"))  // Amazon is often first
        });
    });
}

fn bench_last_match(c: &mut Criterion) {
    // Test a number that likely matches a later courier
    c.bench_function("last_courier_match", |b| {
        b.iter(|| {
            track(black_box("9400111202555842332908"))  // USPS
        });
    });
}

criterion_group!(
    benches,
    bench_valid_numbers,
    bench_invalid_numbers,
    bench_mixed_workload,
    bench_single_track_ups,
    bench_sequential_ups,
    bench_not_found,
    bench_first_match,
    bench_last_match,
);

criterion_main!(benches);
