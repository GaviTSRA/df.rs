use crate::{definitions::{ArgType, DefinedTag}, token::{Position, Selector, Type}};

pub trait Node {
    fn json(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct FileNode {
    pub events: Vec<EventNode>,
    pub functions: Vec<FunctionNode>,
    pub start_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub struct EventNode {
    pub event_type: Option<ActionType>,
    pub event: String,
    pub expressions: Vec<ExpressionNode>,
    pub start_pos: Position,
    pub name_end_pos: Position,
    pub end_pos: Position,
    pub cancelled: bool
}

#[derive(Clone, Debug)]
pub struct FunctionNode {
    pub name: String,
    pub params: Vec<FunctionParamNode>,
    pub expressions: Vec<ExpressionNode>,
    pub start_pos: Position,
    pub name_end_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub struct FunctionParamNode {
    pub name: String,
    pub param_type: Type,
    pub optional: bool,
    pub multiple: bool,
    pub default: Option<ArgValueWithPos>
}

#[derive(Clone, Debug)]
pub struct ExpressionNode {
    pub node: Expression,
    pub start_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub enum Expression {
    Action { node: ActionNode },
    Conditional { node: ConditionalNode },
    Variable { node: VariableNode },
    Call { node: CallNode },
    Repeat { node: RepeatNode }
}

#[derive(Clone, Debug)]
pub struct ActionNode {
    pub action_type: ActionType,
    pub selector: Selector,
    pub name: String,
    pub args: Vec<Arg>,
    pub start_pos: Position,
    pub selector_start_pos: Position,
    pub selector_end_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub struct ConditionalNode {
    pub conditional_type: ConditionalType,
    pub selector: Selector,
    pub name: String,
    pub args: Vec<Arg>,
    pub start_pos: Position,
    pub selector_start_pos: Option<Position>,
    pub selector_end_pos: Option<Position>,
    pub end_pos: Position,
    pub expressions: Vec<ExpressionNode>,
    pub else_expressions: Vec<ExpressionNode>,
    pub inverted: bool
}

#[derive(Clone, Debug)]
pub struct CallNode {
    pub name: String,
    pub args: Vec<Arg>,
    pub start_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub struct RepeatNode {
    pub name: String,
    pub args: Vec<Arg>,
    pub start_pos: Position,
    pub end_pos: Position,
    pub expressions: Vec<ExpressionNode>
}

#[derive(Clone, Debug)]
pub struct Arg {
    pub value: ArgValue,
    pub index: i32,
    pub arg_type: ArgType,
    pub start_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub struct VariableNode {
    pub dfrs_name: String,
    pub df_name: String,
    pub var_type: VariableType,
    pub start_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug)]
pub enum ArgValue {
    Empty,
    Number { number: f32 },
    String { string: String },
    Text { text: String },
    Location { x: f32, y: f32, z: f32, pitch: Option<f32>, yaw: Option<f32> },
    Vector { x: f32, y: f32, z: f32},
    Sound { sound: String, volume: f32, pitch: f32 },
    Potion { potion: String, amplifier: f32, duration: f32 },
    Tag { tag: String, value: String, definition: Option<DefinedTag>, name_end_pos: Position, value_start_pos: Position },
    Variable { name: String, scope: String },
    GameValue { value: String, selector: Selector, selector_end_pos: Position },
    Condition { name: String, args: Vec<Arg>, selector: Selector, conditional_type: ConditionalType, inverted: bool }
}

#[derive(Clone, Debug)]
pub struct ArgValueWithPos {
    pub value: ArgValue,
    pub start_pos: Position,
    pub end_pos: Position
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActionType {
    Player,
    Entity,
    Game,
    Variable,
    Control,
    Select,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConditionalType {
    Player,
    Entity,
    Game,
    Variable
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariableType {
    Line,
    Local,
    Game,
    Save
}