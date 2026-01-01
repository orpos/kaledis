use darklua_core::{
    nodes::{Block, DecimalNumber, Expression, NumberExpression},
    process::{DefaultVisitor, NodeProcessor, NodeVisitor},
    rules::{Context, FlawlessRule, RuleConfiguration, RuleConfigurationError, RuleProperties},
};

pub const REMOVE_NUMBER_LITERALS_MODIFIER_NAME: &str = "remove_number_literals";

struct Processor {}

impl NodeProcessor for Processor {
    fn process_expression(&mut self, exp: &mut Expression) {
        if let Expression::Number(num_exp) = exp {
            match num_exp {
                NumberExpression::Binary(binary) => {
                    let value = binary.compute_value();
                    *exp = DecimalNumber::new(value).into();
                }
                NumberExpression::Decimal(decimal) => {
                    let value = decimal.compute_value();
                    *exp = DecimalNumber::new(value).into();
                }
                _ => {}
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct RemoveNumberLiterals {}

impl FlawlessRule for RemoveNumberLiterals {
    fn flawless_process(&self, block: &mut Block, _: &Context) {
        let mut processor = Processor {};
        DefaultVisitor::visit_block(block, &mut processor);
    }
}

impl RuleConfiguration for RemoveNumberLiterals {
    fn configure(&mut self, _: RuleProperties) -> Result<(), RuleConfigurationError> {
        Ok(())
    }

    fn get_name(&self) -> &'static str {
        REMOVE_NUMBER_LITERALS_MODIFIER_NAME
    }

    fn serialize_to_properties(&self) -> RuleProperties {
        RuleProperties::new()
    }
}
