use std::{env::var, io::{Error, ErrorKind}};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tiny_tokio_actor::{Actor, ActorContext, async_trait, Handler, Message};

use super::event::Event;

/// --------------------------- ///
/// ---------- Actor ---------- ///
/// --------------------------- ///
/// Git actor
#[derive(Clone)]
pub struct Git {
    /// The token used to authenticate to the Gitlab API.
    pub token: String,
    /// The base url of the Gitlab API.
    pub base_url: String
}

/// Git actor implementation.
#[async_trait]
impl Actor<Event> for Git {}

/// Default implementation for the Git actor.
impl Default for Git {
    fn default() -> Self {
        Git {
            token : var("GITLAB_TOKEN").unwrap(),
            base_url : var("GITLAB_URL").unwrap()
        }
    }
}

/// ------------------------------ ///
/// ---------- Messages ---------- ///
/// ------------------------------ ///

/// ---------- Get Project ---------- ///
/// Message used to get the projects from the Gitlab API.
#[derive(Clone)]
pub struct GetProject {
    /// The name of the project to search for.
    pub project_name: String
}

impl Message for GetProject {
    /// The type of the result.
    /// A result that contains either the id of the project or an error.
    type Response = Result<u64, Error>;
}

/// Handler for the GetProjects message for the Git actor.
#[async_trait]
impl Handler<Event, GetProject> for Git {
    async fn handle(&mut self, msg: GetProject, _ctx: &mut ActorContext<Event>) -> Result<u64, Error> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/projects", self.base_url))
            .header("PRIVATE-TOKEN", self.token.clone())
            .query(&[("search", msg.project_name)])
            .send().await;
        match res {
            Ok(res) => {
                let projects: Vec<Project> = res.json().await.unwrap();

                match projects.len() {
                    0 => {
                        return Err(Error::new(ErrorKind::NotFound, "No project found that matches the researched term."));
                    },
                    1 => {
                        return Ok(projects[0].id);
                    },
                    _ => {
                        return Err(Error::new(ErrorKind::Unsupported, "Multiple projects found that matches the researched term."));
                    }
                }
            }
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, "Search request failed."));
            }
        }
    }
}

/// ---------- Get Jobs ---------- ///
#[derive(Clone)]
pub struct GetJobs {
    /// The id of the project to get the jobs from.
    pub project_id: u64,
    /// The date the jobs must be older than.
    pub older_than: DateTime<Utc>,
    /// The page of the jobs to get.
    pub page: u64
}

/// GetJobsResponse structure that holds the response of the GetJobs message.
pub struct GetJobsResponse {
    /// The jobs that were found.
    pub jobs: Vec<Job>,
    /// The next page of jobs to get.
    pub next_page: Option<u64>
}

/// GetJobs message implementation.
impl Message for GetJobs {
    /// The type of the result.
    /// A result that contains either the jobs that were found or an error.
    type Response = Result<GetJobsResponse, Error>;
}

/// Handler for the GetJobs message for the Git actor.
#[async_trait]
impl Handler<Event, GetJobs> for Git {
    async fn handle(&mut self, msg: GetJobs, _ctx: &mut ActorContext<Event>) -> Result<GetJobsResponse, Error> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/projects/{}/jobs", self.base_url, msg.project_id))
            .header("PRIVATE-TOKEN", self.token.clone())
            .query(&[("per_page", "50"), ("page", &msg.page.to_string())])
            .send().await;
        match res {
            Ok(res) => {
                let headers = res.headers().clone();
                let jobs: Vec<Job> = res.json().await.unwrap();
                let next_page = headers
                    .get("x-next-page")
                    .and_then(|x| x.to_str().ok())
                    .and_then(|x| x.parse::<u64>().ok());
                return Ok(GetJobsResponse {
                    jobs,
                    next_page
                });
            }
            Err(err) => {
                return Err(Error::new(ErrorKind::Other, err.to_string()));
            }
        }
    }
}

/// ---------- Erase Job ---------- ///
/// Message used to erase a job from the Gitlab API.
#[derive(Clone)]
pub struct EraseJob {
    /// The id of the project to erase the job from.
    pub project_id: u64,
    /// The id of the job to erase.
    pub job_id: u64
}

/// EraseJob message implementation.
impl Message for EraseJob {
    /// The type of the result.
    /// A result that contains either nothing or an error.
    type Response = Result<(), Error>;
}

/// Handler for the EraseJob message for the Git actor.
#[async_trait]
impl Handler<Event, EraseJob> for Git {
    async fn handle(&mut self, msg: EraseJob, _ctx: &mut ActorContext<Event>) -> Result<(), Error> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/projects/{}/jobs/{}/erase", self.base_url, msg.project_id, msg.job_id))
            .header("PRIVATE-TOKEN", self.token.clone())
            .send().await;
        match res {
            Ok(_) => {
                return Ok(());
            }
            Err(err) => {
                return Err(Error::new(ErrorKind::Other, err.to_string()));
            }
        }
    }
}

/// ---------------------------- ///
/// ---------- Models ---------- ///
/// ---------------------------- ///

/// Project model.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    /// The id of the project.
    pub id: u64,
    /// The name of the project.
    pub name: String,
}

/// Job model.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Job {
    /// The id of the job.
    pub id: u64,
    /// The creation date of the job.
    pub created_at: DateTime<Utc>,
    /// The erase date of the job.
    pub erased_at: Option<DateTime<Utc>>
}

