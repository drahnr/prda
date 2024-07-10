use color_eyre::eyre::*;
use fs_err as fs;
use octocrab::models::pulls::PullRequest;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Config {
    token: String,
    default_base: Option<String>,
}

#[derive(clap::Parser)]
struct Args {
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,
    
    #[command(subcommand)]
    cmd: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Pr {
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        branch: Option<String>,
        #[arg(long)]
        base: Option<String>,
    },
    List {
        // filter: String,
        
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = <Args as clap::Parser>::try_parse()?;
    let pd = directories::ProjectDirs::from("", "io.konifay", "prda").expect("Has home dir. qed");
    
    let config = pd.config_dir().join("config.toml");
    let config = fs::read_to_string(config)?;
    let config: Config = toml::from_str(config.as_str())?;
    
    let repo = gix::open(std::env::current_dir()?)?;
    let branches = repo.branch_names();
    let worktree = repo.worktree();

    // if worktree.kind() == gix::head::Kind::Bare {
    //     bail!("Cannot work with bare repositories");
    // }
    
    dbg!(repo.head_commit()?);
    dbg!(repo.head_ref()?);
    dbg!(repo.head_name()?);
    repo.head()?;
    dbg!(repo.head_id()?);

    let title = "title";
    let branch = "foo";
    let base = "main";
    let owner = "drahnr";
    let repo = "whatever";
    // let (owner, repo) = remote.parse();
    let oc = octocrab::OctocrabBuilder::default().personal_token(config.token).set_connect_timeout(None).build()?;
    let prh = oc.pulls(owner, repo);
    let pr = prh.create(title, branch, base).body("foo").maintainer_can_modify(true).draft(true).send().await?;
    let PullRequest { number, url, id, .. } = pr;
    println!("{number}");
    
    Ok(())
}
