//! TypeScript declaration collection for the bindings generator.

use std::{
    any::TypeId,
    collections::{BTreeMap, HashSet},
};

use ts_rs::{Config, TS, TypeVisitor};

pub struct DeclarationCollector<'a> {
    config: &'a Config,
    seen: HashSet<TypeId>,
    declarations: BTreeMap<String, String>,
}

impl<'a> DeclarationCollector<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            seen: HashSet::new(),
            declarations: BTreeMap::new(),
        }
    }

    pub fn collect<T: TS + 'static + ?Sized>(&mut self) {
        self.visit::<T>();
    }

    pub fn into_declarations(self) -> BTreeMap<String, String> {
        self.declarations
    }
}

impl TypeVisitor for DeclarationCollector<'_> {
    fn visit<T: TS + 'static + ?Sized>(&mut self) {
        if !self.seen.insert(TypeId::of::<T>()) {
            return;
        }

        if T::output_path().is_some() {
            let declaration = normalize_declaration(T::decl(self.config));
            let previous = self
                .declarations
                .insert(T::ident(self.config), declaration.clone());

            assert!(
                previous.as_ref().is_none_or(|value| value == &declaration),
                "conflicting TypeScript declarations for {}",
                T::ident(self.config)
            );
        }

        T::visit_dependencies(self);
        T::visit_generics(self);
    }
}

fn normalize_declaration(declaration: String) -> String {
    let mut declaration = declaration.replace("Record<symbol, never>", "Record<string, never>");
    let prefix = "{ [key in string]: ";

    while let Some(start) = declaration.find(prefix) {
        let value_start = start + prefix.len();
        let end = declaration[value_start..]
            .find(" }")
            .map(|offset| value_start + offset)
            .expect("ts-rs string map declaration should have a closing brace");
        let value = declaration[value_start..end].to_string();

        declaration.replace_range(start..end + 2, &format!("Record<string, {value}>"));
    }

    declaration
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}
