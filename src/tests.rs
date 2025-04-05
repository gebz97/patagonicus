use scout::units;

fn main() {
    let max_int = u64::MAX;
    println!("{0}", units::human_readable_iec(max_int));
}