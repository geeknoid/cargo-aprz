use super::common::{Common, CommonArgs};
use crate::Result;
use crate::facts::CrateRef;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct CratesArgs {
    /// Crates to appraise (format: `crate_name` or `crate_name@version`)
    #[arg(value_name = "CRATE")]
    pub crates: Vec<CrateRef>,

    #[command(flatten)]
    pub common: CommonArgs,
}

pub async fn process_crates(args: &CratesArgs) -> Result<()> {
    let common = Common::new(&args.common).await?;
    let crate_facts = common.process_crates(args.crates.clone(), true).await?;

    common.report(crate_facts.into_iter())
}
