pub mod agile_bugs;
pub mod agile_epics;
pub mod agile_features;
pub mod agile_participants;
pub mod agile_projects;
pub mod agile_stories;
pub mod agile_tasks;
pub mod agile_workitems;
pub mod directory_users;
pub mod op_executor;

use crate::configure::OpContext;
use crate::wt_error::WTError;
use crate::AnyError;
use clap::ArgMatches;

pub struct OpExecutors {
    executors: std::collections::HashMap<String, Box<dyn op_executor::OpExecutor>>,
}

impl OpExecutors {
    pub fn initialize() -> OpExecutors {
        let mut es: std::collections::HashMap<String, Box<dyn op_executor::OpExecutor>> =
            std::collections::HashMap::new();
        es.insert(
            String::from("agile_projects_list"),
            Box::new(agile_projects::AgileProjectListOpExecutor {}),
        );
        es.insert(
            String::from("agile_projects_get"),
            Box::new(agile_projects::AgileProjectGetOpExecutor {}),
        );
        es.insert(
            String::from("directory_users_get"),
            Box::new(directory_users::DirectoryUsersGetOpExecutor {}),
        );
        es.insert(
            String::from("directory_users_list"),
            Box::new(directory_users::DirectoryUsersListOpExecutor {}),
        );
        es.insert(
            String::from("agile_epics_create"),
            Box::new(agile_epics::AgileEpicCreateOpExecutor {}),
        );
        es.insert(
            String::from("agile_features_create"),
            Box::new(agile_features::AgileFeaturesCreateOpExecutor {}),
        );
        es.insert(
            String::from("agile_stories_create"),
            Box::new(agile_stories::AgileStoriesCreateOpExecutor {}),
        );
        es.insert(
            String::from("agile_tasks_create"),
            Box::new(agile_tasks::AgileTasksCreateOpExecutor {}),
        );
        es.insert(
            String::from("agile_bugs_create"),
            Box::new(agile_bugs::AgileBugsCreateOpExecutor {}),
        );
        es.insert(
            String::from("agile_workitems_list"),
            Box::new(agile_workitems::AgileWorkitemsListOpExecutor {}),
        );
        es.insert(
            String::from("agile_workitems_delete"),
            Box::new(agile_workitems::AgileWorkitemsDeleteOpExecutor {}),
        );
        es.insert(
            String::from("agile_epics_update"),
            Box::new(agile_epics::AgileEpicUpdateOpExecutor {}),
        );
        es.insert(
            String::from("agile_participants_list"),
            Box::new(agile_participants::AgileParticipantsListOpExecutor {}),
        );
        es.insert(
            String::from("agile_participants_add"),
            Box::new(agile_participants::AgileParticipantsAddOpExecutor {}),
        );
        OpExecutors { executors: es }
    }

    pub fn execute(&self, ctx: &OpContext, matches: &ArgMatches) -> Result<(), AnyError> {
        if let Some(executor) = self.executors.get(&ctx.key) {
            executor.execute(matches, ctx)
        } else {
            Err(WTError::new_boxed(
                "000000",
                &format!("Cannot find op through {}", ctx.key),
            ))
        }
    }
}
