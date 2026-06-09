use std::collections::BTreeMap;
use std::fs::{create_dir_all, write};

use eyre::{Context, Result};

use crate::config::{AppConfig, ModuleLocation};
use crate::module::Module;

pub fn sync(config: AppConfig) -> Result<()> {
    // collect modules
    let mut module_invocations = config.modules.clone();
    let mut modules = Vec::with_capacity(module_invocations.len());

    while let Some(invocation) = module_invocations.pop() {
        // TODO: this does not disallow recursion in any way. A module could
        // call itself, or two modules could call each other in a loop. Tracking
        // this and exiting cleanly with an error would be good, but not
        // essential before we get the rest of the system working.

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

    // read existing files
    for module in &mut modules {
        module.collect_splices()?; // TODO: contextualize with name as below
    }

    // generate new files, syncing in splice blocks
    let mut out = BTreeMap::new();
    for module in modules {
        // Note that overlapping paths will be overwritten by later modules.
        // This is by design. This program will grow a masking functionality
        // sometime later to deal with exceptions here.
        //
        // TODO: add context about which module failed. Needs to have path/name
        // added to each module before that can happen.
        out.append(&mut module.files()?);
    }

    // TODO: calculate diff
    // TODO: present a diff to the user (?)

    // write files
    for (path, contents) in out.iter() {
        // TODO: only write changed files. Print a log with unchanged ones.
        tracing::info!(file = ?path, "writing");
        if let Some(dir) = path.parent() {
            create_dir_all(dir)
                .wrap_err_with(|| format!("Could not create `{}`", dir.display()))?;
        }

        write(path, contents).wrap_err_with(|| format!("Could not write `{}`", path.display()))?;
    }

    Ok(())
}
