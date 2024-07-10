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
    let Args { cmd, config, } = args;
    
    let config_path = if let Some(config_path) = config {
        config_path
    } else {
        let pd = directories::ProjectDirs::from("", "io.konifay", "prda").expect("Has home dir. qed");
        let config_path = pd.config_dir().join("config.toml");
        config_path            
    };
    let config = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(config.as_str())?;
    
    let repo = gix::open(std::env::current_dir()?)?;
    let branches = repo.branch_names();
    let worktree = repo.worktree();

    // if worktree.kind() == gix::head::Kind::Bare {
    //     bail!("Cannot work with bare repositories");
    // }
    
    match cmd {
        Command::Pr { title, branch, base } => {
            
            let base = base.as_ref().map(|s| s.as_str()).unwrap_or_else(|| config.default_base.as_ref().map(|s| s.as_str()).unwrap_or("main"));
            dbg!(repo.head_commit()?);
            let Some(head_name) = repo.head_name()?else {
                bail!("Must be on a named branch, otherwise this won't work")
            };
            let branch = head_name.shorten();

            let mut iter = repo.remote_names().into_iter().filter_map(|remote| repo.try_find_remote(&*remote));
            let (owner, repo) = loop {
                let Some(remote) = iter.next() else {
                    bail!("Didn't find a remote to deduce owner and repo from..")
                };
                let remote = remote?;
                let Some(remote) = remote.url(gix::remote::Direction::Push) else {
                    bail!("Failed to find remote (push) url");
                };
                let re = regex::Regex::new(r#"^(git@|https://)github.com[:/](?<owner>[a-zA-z0-9]+)/(?<repo>[a-zA-z0-9]+)(\.git)?$"#).expect("Regex is sane. qed");
                let haystack = remote.to_string();
                if let Some(captures) = re.captures(&haystack) {
                    let owner = captures.name("owner").expect("Always present. qed");
                    let repo = captures.name("repo").expect("Always present. qed");
                    break dbg!((owner.as_str().to_owned(), repo.as_str().to_owned()));
                }
            };

            let oc = octocrab::OctocrabBuilder::default().personal_token(config.token).set_connect_timeout(None).build()?;
            let prh = oc.pulls(owner, repo);
            let pr = prh.create(title, dbg!(branch.to_string()), dbg!(base)).body("foo").maintainer_can_modify(true).draft(true).send().await?;
            let PullRequest { number, url, id, .. } = pr;
            println!("{number}");

        }
         _ => todo!()
    }
    let branch = "foo";
    let base = "main";
    
    Ok(())
}
