fn main() {
    println!(
        "cargo:rustc-link-search=native={}",
        "../samplecount/methcla/build/src"
    );
}
