use std::{fmt::{Display, self, Formatter}, io::Error};

use chrono::{Utc, DateTime};
use clap::{Parser, ValueEnum};

mod actors;
use actors::{displ::Displ, git::{Git, GetProject, GetJobs, Job}, event::Event};
use tiny_tokio_actor::{EventBus, ActorSystem, ActorRef};

/// Enum used to define the target component(s) of the project to clean.
#[derive(Parser, Debug, Clone, ValueEnum)]
enum Target {
    /// The target is the jobs of the project.
    Jobs,
}

impl Display for Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Target::Jobs => write!(f, "jobs"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    /// The name of the project to search for.
    #[arg(short, long)]
    project: String,

    /// The target component(s) of the project to clean.
    #[clap(value_enum)]
    #[arg(short, long, default_value = "jobs")]
    target: Target, 

    /// The expiration date of the component(s) to clean.
    #[arg(value_parser = parse_duration, default_value = "100")]
    expiration_in_days: std::time::Duration,
}

/// Parse a duration from a days count.
fn parse_duration(arg: &str) -> Result<std::time::Duration, std::num::ParseIntError> {
    let days : u64 = arg.parse()?;
    Ok(std::time::Duration::from_secs(60 * 60 * 24 * days))
}

#[tokio::main]
async fn main() {
    // Init the actor system.
    let bus = EventBus::<Event>::new(1000);
    let system = ActorSystem::new("gitlab-cleaner", bus);

    // Getting the arguments from the CLI parser
    let args = Args::parse();
    let project_name = args.project;
    let expiration_date = chrono::Utc::now() - args.expiration_in_days;

    let displ = Displ::default();
    let git = Git::default();
    let git_ref = system.create_actor("git-actor", git).await.unwrap();
    let displ_ref = system.create_actor("displ-actor", displ).await.unwrap();

    let get_project_message = GetProject {
        project_name: project_name.clone()
    };
    
    // Better unwrap here to panic in case of error.
    let project_id = git_ref.ask(get_project_message).await
        .or_else(|err| Err(Error::new(std::io::ErrorKind::Other, err.to_string())))
        .or_else(|err| Err(Error::new(std::io::ErrorKind::Other, err.to_string())))
        .unwrap().unwrap();

    match args.target {
        Target::Jobs => clean_jobs(&git_ref, &displ_ref, project_id, expiration_date).await,
    }

}

async fn clean_jobs(
    git_ref: &ActorRef<Event, Git>,
    displ_ref: &ActorRef<Event, Displ>, 
    project_id: u64, 
    expiration_date: DateTime<Utc>) -> () {
    let mut jobs_page = Some(1);
    let mut full_jobs: Vec<Job> = Vec::new();
    while let Some(page) = jobs_page {
        let _ = displ_ref.ask(actors::displ::DisplayMessage {
            message: format!("Loading jobs from page {}", page)
        }).await;

        let jobs_result = git_ref.ask(GetJobs {
            project_id,
            older_than: expiration_date,
            page
        }).await
            .or(Err(Error::new(std::io::ErrorKind::Other, "Could not send the action to get the jobs.")))
            .or(Err(Error::new(std::io::ErrorKind::Other, "Could not find the jobs.")))
            .unwrap().unwrap();

        full_jobs.append(jobs_result.jobs.clone().as_mut());
        jobs_page = jobs_result.next_page;
    };

    let jobs_count: u64 = full_jobs.len() as u64;

    let _ = displ_ref.ask(actors::displ::DisplayMessage {
        message: format!("Found {} jobs to clean.", jobs_count)
    }).await;
    displ_ref.ask(actors::displ::InitProgressBar {
        length: jobs_count,
        message: format!("Cleaning the jobs...")
    }).await
        .or(Err(Error::new(std::io::ErrorKind::Other, "Could not prepare the progress bar somehow."))).unwrap();

    let future_results = full_jobs.iter().map(|job| async {
        git_ref.ask(actors::git::EraseJob {
            project_id,
            job_id: job.id
        }).await
            .or(Err(Error::new(std::io::ErrorKind::Other, format!("Could not send the action to erase the job {}", job.id))))?
            .or(Err(Error::new(std::io::ErrorKind::Other, format!("Could not erase the job {}", job.id))))?;

        let _ = displ_ref.ask(actors::displ::IncreaseProgress {
            message: format!("Job {} erased.", job.id)
        }).await;
        Ok(())
    });

    let results: Vec<Result<(), Error>> = futures::future::join_all(future_results).await;
    results.iter().filter(|r| r.is_err()).for_each(|r| {
        println!("Error: {}", r.as_ref().unwrap_err());
    });

    let _ = displ_ref.ask(actors::displ::DisplayMessage {
        message: "Done erasing jobs." .to_string()
    }).await;
}

