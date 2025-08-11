use std::sync::Arc;

use chrono::Utc;
use colette_core::{
    Handler as _,
    job::{
        CreateJobCommand, CreateJobHandler, GetJobHandler, GetJobQuery, Job, JobStatus,
        UpdateJobCommand, UpdateJobError, UpdateJobHandler,
    },
};
use colette_job::{Error, JobError};
use colette_queue::JobConsumer;
use cron::Schedule;
use serde_json::Value;
use tower::{Service, ServiceExt, util::BoxService};

pub struct JobWorker {
    get_job: Arc<GetJobHandler>,
    update_job: Arc<UpdateJobHandler>,
    job_consumer: Box<dyn JobConsumer>,
    job_handler: BoxService<Job, (), Error>,
}

impl JobWorker {
    pub fn new(
        get_job: Arc<GetJobHandler>,
        update_job: Arc<UpdateJobHandler>,
        job_consumer: impl JobConsumer,
        job_handler: BoxService<Job, (), Error>,
    ) -> Self {
        Self {
            get_job,
            update_job,
            job_consumer: Box::new(job_consumer),
            job_handler,
        }
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        while let Some(job_id) = self.job_consumer.pop().await? {
            let job = match self
                .update_job
                .handle(UpdateJobCommand {
                    id: job_id.into(),
                    data: None,
                    status: Some(JobStatus::Running),
                    message: None,
                })
                .await
            {
                Ok(_) => match self.get_job.handle(GetJobQuery { id: job_id.into() }).await {
                    Ok(job) => job,
                    Err(e) => return Err(Error::Job(JobError::GetJob(e))),
                },
                Err(UpdateJobError::AlreadyCompleted(_)) => {
                    continue;
                }
                Err(e) => return Err(Error::Job(JobError::UpdateJob(e))),
            };

            match self.job_handler.ready().await?.call(job).await {
                Ok(_) => self
                    .update_job
                    .handle(UpdateJobCommand {
                        id: job_id.into(),
                        data: None,
                        status: Some(JobStatus::Completed),
                        message: None,
                    })
                    .await
                    .map_err(JobError::UpdateJob)?,
                Err(e) => self
                    .update_job
                    .handle(UpdateJobCommand {
                        id: job_id.into(),
                        data: None,
                        status: Some(JobStatus::Failed),
                        message: Some(Some(e.to_string())),
                    })
                    .await
                    .map_err(JobError::UpdateJob)?,
            };
        }

        Ok(())
    }
}

pub struct CronWorker {
    name: String,
    schedule: Schedule,
    get_job: Arc<GetJobHandler>,
    create_job: Arc<CreateJobHandler>,
    update_job: Arc<UpdateJobHandler>,
    handler: BoxService<Job, (), Error>,
}

impl CronWorker {
    pub fn new<S: Into<String>>(
        name: S,
        schedule: Schedule,
        get_job: Arc<GetJobHandler>,
        create_job: Arc<CreateJobHandler>,
        update_job: Arc<UpdateJobHandler>,
        handler: BoxService<Job, (), Error>,
    ) -> Self {
        Self {
            name: name.into(),
            schedule,
            get_job,
            create_job,
            update_job,
            handler,
        }
    }

    pub async fn start(&mut self) {
        loop {
            let upcoming = self.schedule.upcoming(Utc).take(1).next().unwrap();

            let duration = (upcoming - Utc::now()).to_std().unwrap();

            tokio::time::sleep(duration).await;

            let job = match self
                .create_job
                .handle(CreateJobCommand {
                    job_type: self.name.clone(),
                    data: Value::Null,
                    group_identifier: None,
                })
                .await
            {
                Ok(id) => match self.get_job.handle(GetJobQuery { id }).await {
                    Ok(job) => job,
                    Err(e) => {
                        tracing::error!("{}", e);
                        continue;
                    }
                },
                Err(e) => {
                    tracing::error!("{}", e);
                    continue;
                }
            };

            let job_id = job.id;

            match self.handler.ready().await.unwrap().call(job).await {
                Ok(_) => {
                    if let Err(e) = self
                        .update_job
                        .handle(UpdateJobCommand {
                            id: job_id,
                            data: None,
                            status: Some(JobStatus::Completed),
                            message: None,
                        })
                        .await
                    {
                        tracing::error!("{}", e);
                    };
                }
                Err(e) => {
                    if let Err(e) = self
                        .update_job
                        .handle(UpdateJobCommand {
                            id: job_id,
                            data: None,
                            status: Some(JobStatus::Failed),
                            message: Some(Some(e.to_string())),
                        })
                        .await
                    {
                        tracing::error!("{}", e);
                    };
                }
            };
        }
    }
}
