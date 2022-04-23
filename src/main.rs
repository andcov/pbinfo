use pbinfo::*;

fn main() {
    let pb = PbInfoProblem::fetch_problem_by_id(877);
    println!("{:#?}", pb);
}
