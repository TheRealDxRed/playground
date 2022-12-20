add_requires("cargo::rust-3d", {configs={cargo_toml=path.join(os.scriptdir(), "Cargo.toml"),version="*"}})
add_requires("libsdl")

target("rust-3d")
	set_kind("binary")
	add_files("src/main.rs")
	add_packages("cargo::rust-3d")
	add_packages("libsdl")