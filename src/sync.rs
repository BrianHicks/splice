use eyre::Result;

use crate::{
    config::{AppConfig, ModuleLocation},
    module::Module,
};

pub fn sync(config: AppConfig) -> Result<()> {
    // collect modules
    let mut module_invocations = config.modules.clone();
    let mut modules = Vec::with_capacity(module_invocations.len());

    while let Some(invocation) = module_invocations.pop() {
        // TODO: this is where a fancier/remote loader would go. Load from HTTP
        // or Git and cache by etag etc. (Maybe need to take care of these in
        // parallel then?)
        let module = match invocation.location {
            ModuleLocation::Local { path } => {
                Module::from_dir(&path, invocation.args, invocation.prefix.clone())?
            }
        };

        module_invocations.extend(module.config.modules.iter().map(|sub_invocation| {
            match &invocation.prefix {
                Some(prefix) => sub_invocation.inherit_prefix(prefix),
                None => sub_invocation.clone(),
            }
        }));

        modules.push(module)
    }

    println!("{modules:#?}");

    Ok(())
}
