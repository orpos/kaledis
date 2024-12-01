use std::sync::{ Arc, Mutex };

use darklua_core::{
    nodes::{ Block, FieldExpression, Prefix },
    process::{ DefaultVisitor, NodeProcessor, NodeVisitor },
    rules::{ Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties },
};
use indexmap::IndexSet;

pub const GET_LOVE_MODIFIER_NAME: &str = "get_love";

struct Processor {
    modules: Arc<Mutex<IndexSet<String>>>,
}

impl NodeProcessor for Processor {
    fn process_field_expression(&mut self, data: &mut FieldExpression) {
        if let Prefix::Identifier(layer1) = data.get_prefix() {
            if layer1.get_name() == "love" {
                let module = data.get_field().get_name();
                self.modules.lock().unwrap().insert(module.clone());
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct GetLoveModules {
    pub modules: Arc<Mutex<IndexSet<String>>>,
}

impl FlawlessRule for GetLoveModules {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let mut processor = Processor {
            modules: Arc::clone(&self.modules),
        };
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for GetLoveModules {
    fn configure(&mut self, _: RuleProperties) -> Result<(), RuleConfigurationError> {
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        GET_LOVE_MODIFIER_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}
