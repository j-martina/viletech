use std::path::Path;

use znbx::sys;

criterion::criterion_group!(benches, end_to_end);
criterion::criterion_main!(benches);

fn end_to_end(crit: &mut criterion::Criterion) {
	let mut grp = crit.benchmark_group("End-to-End");

	grp.sample_size(10);

	let Ok(sample) = std::env::var("ZNBX_SAMPLE_WAD") else {
		eprintln!("Env. var. `ZNBX_SAMPLE_WAD` not set; skipping benchmarks.");
		return;
	};

	grp.bench_function("Vanilla", |bencher| {
		let path = Path::new(&sample);
		let bytes = std::fs::read(path).unwrap();

		bencher.iter(|| unsafe {
			let reader = sys::znbx_wadreader_new(bytes.as_ptr());
			let p = sys::znbx_processor_new(reader, std::ptr::null());
			sys::znbx_processor_run(p, std::ptr::null());
			sys::znbx_processor_destroy(p);
			sys::znbx_wadreader_destroy(reader);
		});
	});

	grp.finish();
}
