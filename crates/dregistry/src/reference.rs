use pest_consume::{match_nodes, Parser};

use crate::digest::Digest;
use crate::error::DResult;
use crate::source::DockerSource;

#[derive(pest_consume::Parser)]
#[grammar = "reference.pest"]
pub struct DockerTagParser;

#[derive(Default)]
struct Prefix {
    host: Option<String>,
    port: Option<u16>,
}

type Result<T> = std::result::Result<T, pest_consume::Error<Rule>>;
pub type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[allow(non_snake_case, clippy::unnecessary_wraps)]
#[pest_consume::parser]
impl DockerTagParser {
    fn hostname(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn port(input: Node) -> Result<u16> {
        Ok(input.as_str().parse().map_err(|e| input.error(e))?)
    }

    fn name_component(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn name_components(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn prefix(input: Node) -> Result<Prefix> {
        match_nodes!(
            input.into_children();
            [] => Ok(Prefix { host: None, port: None }),
            [hostname(host)] => Ok(Prefix { host: Some(host), port: None }),
            [hostname(host), port(port)] => Ok(Prefix { host: Some(host), port: Some(port) }),
        )
    }

    fn image_name(input: Node) -> Result<(Prefix, Option<String>, String)> {
        let (prefix, names) = match_nodes!(
            input.into_children();
            [name_components(names)] => (Prefix::default(), names),
            [prefix(pf), name_components(names)] => (pf, names),
        );

        if let Some((head, tail)) = names.rsplit_once('/') {
            Ok((prefix, Some(head.to_string()), tail.to_string()))
        } else {
            Ok((prefix, None, names))
        }
    }

    fn tag_name(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn digest(input: Node) -> Result<Digest> {
        Ok(Digest::parse(input.as_str()).map_err(|e| input.error(e))?)
    }

    fn container(input: Node) -> Result<DockerSource> {
        match_nodes!(
            input.into_children();

            [image_name((prefix, namespace, repository))] => Ok(DockerSource {
                host: prefix.host,
                port: prefix.port,
                namespace,
                repository,
                tag: None,
                digest: None,
            }),

            [image_name((prefix, namespace, repository)), tag_name(tn)] => Ok(DockerSource {
                host: prefix.host,
                port: prefix.port,
                namespace,
                repository,
                tag: Some(tn),
                digest: None,
            }),

            [image_name((prefix, namespace, repository)), digest(dg)] => Ok(DockerSource {
                host: prefix.host,
                port: prefix.port,
                namespace,
                repository,
                tag: None,
                digest: Some(dg),
            }),
        )
    }

    fn EOI(input: Node) -> Result<()> {
        Ok(())
    }

    fn DOCKER_REFERENCE(input: Node) -> Result<DockerSource> {
        match_nodes!(
            input.into_children();
            [container(ctr), _EOI] => Ok(ctr)
        )
    }
}

pub fn parse(input: &str) -> DResult<DockerSource> {
    let inputs = DockerTagParser::parse(Rule::DOCKER_REFERENCE, input)?;
    let input = inputs.single()?;
    Ok(DockerTagParser::DOCKER_REFERENCE(input)?)
}
