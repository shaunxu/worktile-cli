use crate::args::ArgParser;
use crate::configure::OpContext;
use crate::op_executors::op_executor::OpExecutor;
use crate::op_executors::op_executor::OpRequest;
use crate::AnyError;
use clap::ArgMatches;

pub struct AgileBugsCreateOpExecutor {}
impl OpExecutor for AgileBugsCreateOpExecutor {
    fn on_execute<'a>(&self, matches: &'a ArgMatches, _context: &OpContext) -> Result<OpRequest<'a>, AnyError> {
        Ok(OpRequest {
            method: reqwest::Method::POST,
            param: None,
            query: None,
            body: ArgParser::parse_content_to_json(matches)?,
            parents: None,
        })
    }
}

pub struct AgileBugsUpdateOpExecutor {}
impl OpExecutor for AgileBugsUpdateOpExecutor {
    fn on_execute<'a>(&self, matches: &'a ArgMatches, _context: &OpContext) -> Result<OpRequest<'a>, AnyError> {
        Ok(OpRequest {
            method: reqwest::Method::PATCH,
            param: matches.value_of("id"),
            query: None,
            body: ArgParser::parse_content_to_json(matches)?,
            parents: None,
        })
    }
}
