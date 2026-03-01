use super::super::helpers::setup_make_v2;

#[test]
fn test_make() {
    let amount_to_receive = 100_000_000u64;
    let amount_to_give = 500_000_000u64;

    let setup = setup_make_v2(amount_to_receive, amount_to_give);

    println!("{:<12} | {:>6} CUs", "make v2", setup.make_cu);
}
