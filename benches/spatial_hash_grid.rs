use bevy_3d::spatial_hash_grid::SpatialHashGrid;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bevy::prelude::*;

fn setup() -> SpatialHashGrid {
    let size = 35;
    let size_half = size / 2;
    let spacing = 5;

    let mut grid = SpatialHashGrid::new(5);
    let mut id = 0;

    for x in -size_half..size_half {
        for y in -size_half..size_half {
            for z in -size_half..size_half {
                grid.update(Entity::from_raw(id), Vec3::new((x*spacing) as f32, (y*spacing) as f32, (z*spacing) as f32));
                id += 1;
            }
        }
    }

    grid
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("get_nearby", |b| {
        let grid = setup();

        b.iter(|| grid.get_nearby(black_box(Vec3::ZERO), 5.0));
    });

    c.bench_function("get_nearby_naive", |b| {
        let grid = setup();

        b.iter(|| grid.get_nearby(black_box(Vec3::ZERO), 5.0));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);