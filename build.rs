fn main() {
	println!("cargo:rerun-if-changed=resources/dreamseeker-dezombifier-manifest.rc");
	println!("cargo:rerun-if-changed=resources/dreamseeker-dezombifier.exe.manifest");
	embed_resource::compile(
		"resources/dreamseeker-dezombifier-manifest.rc",
		embed_resource::NONE,
	);
}
