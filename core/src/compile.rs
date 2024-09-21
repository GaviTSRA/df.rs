use std::fmt;
use serde::{de, ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, Visitor};
use crate::{node::{ActionNode, ActionType, CallNode, ConditionalNode, ConditionalType, EventNode, Expression, FileNode, FunctionNode, RepeatNode}, token::{get_type_str, Selector}};

pub fn compile(node: FileNode, debug: bool) -> Vec<CompiledLine> {
    let mut res: Vec<CompiledLine> = vec![];
    for function in node.functions.clone() {
        match function_node(function.clone()) {
            Ok(result) => {
                res.push(CompiledLine {
                    name: format!("Function {}", function.name),
                    code: result.clone()
                });
                if debug {
                    println!("{:?}", result);
                }
            }
            Err(err) => {
                panic!("Failed to compile: {}", err)
            }
        }
    }
    for event in node.events.clone() {
        match event_node(event.clone()) {
            Ok(result) => {
                res.push(CompiledLine {
                    name: format!("Event {}", event.event),
                    code: result.clone()
                });
                if debug {
                    println!("{:?}", result);
                }
            }
            Err(err) => {
                panic!("Failed to compile: {}", err)
            }
        }
    }
    res
}

fn event_node(event_node: EventNode) -> Result<String, serde_json::Error> {
    let mut codeline = Codeline { blocks: vec![] };

    let attribute = if event_node.cancelled {
        Some("LS-CANCEL".into())
    } else {
        None
    };

    let event_block = Block {
        id: "block".to_owned(), 
        sub_action: None,
        block: if event_node.event_type.unwrap() == ActionType::Player { Some("event".to_owned()) } else { Some("entity_event".to_owned()) }, 
        action: Some(event_node.event),
        args: Some(Args { items: vec![] }),
        target: None,
        data: None,
        direct: None,
        bracket_type: None,
        attribute
    };
    codeline.blocks.push(event_block);

    for expr_node in event_node.expressions {
        if let Some(blocks) = expression_node(expr_node.node) {
            for block in blocks {
                codeline.blocks.push(block);
            }
        }
    }

    let res = serde_json::to_string(&codeline)?;

    Ok(res)
}

fn function_node(function_node: FunctionNode) -> Result<String, serde_json::Error> {
    let mut codeline = Codeline { blocks: vec![] };

    let mut items = vec![
        Arg { item: ArgItem { data: ArgValueData::Id { id: "function".into() }, id: "hint".into() }, slot: 25 },
        Arg { item: ArgItem { data: ArgValueData::Tag { action: "dynamic".into(), block: "func".into(), option: "False".into(),tag: "Is Hidden".into() }, id: "bl_tag".into() }, slot: 26 }
    ];

    for (slot, param) in function_node.params.into_iter().enumerate() {
        let mut default = None;
        if let Some(param_default) = param.default {
            let default_data = arg_val_from_arg(crate::node::Arg {
                value: param_default.value,
                index: 0,
                arg_type: crate::definitions::ArgType::ANY,
                start_pos: param_default.start_pos,
                end_pos: param_default.end_pos,
            }, "".into(), "".into()).unwrap().item;
            
            default = Some(FunctionDefaultItem {
                data: match default_data.data {
                    ArgValueData::Simple { name } => FunctionDefaultItemData::Simple { name },
                    ArgValueData::Id { id } => FunctionDefaultItemData::Id { id },
                    ArgValueData::Location { is_block, loc } => FunctionDefaultItemData::Location { is_block, loc },
                    ArgValueData::Vector { x, y, z } => FunctionDefaultItemData::Vector { x, y, z },
                    ArgValueData::Sound { sound, volume, pitch } => FunctionDefaultItemData::Sound { sound, volume, pitch },
                    ArgValueData::Potion { potion, amplifier, duration } => FunctionDefaultItemData::Potion { potion, amplifier, duration },
                    _ => unreachable!()
                },
                id: default_data.id,
            })
        }
        
        items.push(Arg {
            item: ArgItem {
                data: ArgValueData::FunctionParam {
                    default_value: default,
                    name: param.name,
                    optional: param.optional,
                    plural: param.multiple,
                    param_type: get_type_str(param.param_type),
                },
                id: "pn_el".into(),
            },
            slot: slot as i32
        });
    }

    let function_block = Block {
        id: "block".to_owned(), 
        block: Some("func".to_owned()),
        attribute: None,
        action: None,
        args: Some(Args { items }),
        target: None,
        data: Some(function_node.name),
        sub_action: None,
        direct: None,
        bracket_type: None
    };
    codeline.blocks.push(function_block);

    for expr_node in function_node.expressions {
        if let Some(blocks) = expression_node(expr_node.node) { 
            for block in blocks {
                codeline.blocks.push(block)
            }
        }
    }

    let res = serde_json::to_string(&codeline)?;

    Ok(res)
}

fn expression_node(node: Expression) -> Option<Vec<Block>> {
    match node {
        Expression::Action { node } => Some(vec![action_node(node)]),
        Expression::Conditional { node } => Some(conditional_node(node)),
        Expression::Call { node } => Some(vec![call_node(node)]),
        Expression::Repeat { node } => Some(repeat_node(node)),
        Expression::Variable { .. } => None,
    }
}

fn conditional_node(node: ConditionalNode) -> Vec<Block> {
    let block = match node.conditional_type {
        ConditionalType::Player => "if_player",
        ConditionalType::Entity => "if_entity",
        ConditionalType::Game => "if_game",
        ConditionalType::Variable => "if_var"
    };

    let mut args: Vec<Arg> = vec![];

    for arg in node.args {
        let arg = match arg_val_from_arg(arg, node.name.clone(), block.to_owned()) {
            Some(res) => res,
            None => continue
        };
        args.push(arg);
    }

    let attribute = if node.inverted {
        Some("NOT".into())
    } else {
        None
    };

    let mut blocks = vec![
        Block {
            action: Some(node.name),
            block: Some(block.to_string()),
            id: "block".to_string(),
            target: match node.conditional_type {
                ConditionalType::Game => None,
                ConditionalType::Variable => None,
                _ => Some(node.selector)
            },
            args: Some(Args { items: args }),
            attribute,
            data: None,
            direct: None,
            bracket_type: None,
            sub_action: None,
        },
        Block {
            id: "bracket".into(),
            direct: Some("open".into()),
            bracket_type: Some("norm".into()),
            block: None,
            attribute: None,
            args: None, 
            action: None,
            sub_action: None,
            target: None, 
            data: None
        },
    ];

    for expression in node.expressions {
        if let Some(expression_blocks) = expression_node(expression.node) {
            for block in expression_blocks {
                blocks.push(block);
            }
        }
    }

    blocks.push(Block {
        id:"bracket".into(),
        direct: Some("close".into()),
        bracket_type: Some("norm".into()), 
        block: None, 
        args: None, 
        action: None,
        target: None, 
        data: None,
        sub_action: None,
        attribute: None
    });

    if !node.else_expressions.is_empty() {
        blocks.push(Block {
            id: "block".into(),
            direct: None,
            bracket_type: None,
            block: Some("else".into()),
            attribute: None,
            args: None,
            sub_action: None,
            action: None,
            target: None,
            data: None
        });
        blocks.push(Block {
            id: "bracket".into(),
            direct: Some("open".into()),
            bracket_type: Some("norm".into()),
            block: None,
            attribute: None,
            args: None,
            action: None,
            target: None,
            sub_action: None,
            data: None
        });

        for expression in node.else_expressions {
            if let Some(expression_blocks) = expression_node(expression.node) {
                for block in expression_blocks {
                    blocks.push(block);
                }
            }
        }

        blocks.push(Block {
            id:"bracket".into(),
            direct: Some("close".into()),
            bracket_type: Some("norm".into()),
            block: None,
            args: None,
            action: None,
            target: None,
            sub_action: None,
            data: None,
            attribute: None
        });
    }

    blocks
}

fn call_node(node: CallNode) -> Block {
    let mut args: Vec<Arg> = vec![];

    for arg in node.args {
        let arg = match arg_val_from_arg(arg, node.name.clone(), "".to_owned()) {
            Some(res) => res,
            None => continue
        };
        args.push(arg);
    }

    Block {
        id: "block".into(),
        block: Some("call_func".into()),
        args: Some(Args { items: args }),
        action: None,
        target: None,
        data: Some(node.name),
        attribute: None,
        direct: None,
        sub_action: None,
        bracket_type: None,
    }
}

fn repeat_node(node: RepeatNode) -> Vec<Block> {
    let mut args: Vec<Arg> = vec![];
    let mut attribute = None;
    let mut sub_action = None;
    let mut target = None;

    if !node.clone().args.is_empty() {
        let arg =  node.args.get(0).clone().unwrap();
        match arg.value.clone() {
            crate::node::ArgValue::Condition { name, args: new_args, selector, inverted, .. } => {
                for arg in new_args {
                    let arg = match arg_val_from_arg(arg, node.name.clone(), "repeat".to_owned()) {
                        Some(res) => res,
                        None => continue
                    };
                    args.push(arg);
                }
                attribute = if inverted {
                    Some("NOT".into())
                } else {
                    None
                };
                sub_action = Some(name);
                target = Some(selector);
            }
            _ => {
                for arg in node.args {
                    let arg = match arg_val_from_arg(arg, node.name.clone(), "repeat".to_owned()) {
                        Some(res) => res,
                        None => continue
                    };
                    args.push(arg);
                }
            }
        }
    }

    let mut blocks = vec![
        Block {
            action: Some(node.name),
            block: Some("repeat".into()),
            id: "block".to_string(),
            target,
            args: Some(Args { items: args }),
            attribute,
            data: None,
            direct: None,
            sub_action: sub_action,
            bracket_type: None
        },
        Block {
            id: "bracket".into(),
            direct: Some("open".into()),
            bracket_type: Some("repeat".into()),
            block: None,
            attribute: None,
            args: None, 
            action: None,
            sub_action: None,
            target: None, 
            data: None
        }
    ];

    for expression in node.expressions {
        if let Some(expression_blocks) = expression_node(expression.node) {
            for block in expression_blocks {
                blocks.push(block);
            }
        }
    }

    blocks.push(Block {
        id:"bracket".into(),
        direct: Some("close".into()),
        bracket_type: Some("repeat".into()), 
        block: None, 
        args: None, 
        action: None,
        target: None, 
        data: None,
        sub_action: None,
        attribute: None
    });

    blocks
}

fn action_node(node: ActionNode) -> Block {
    let block = match node.action_type {
        ActionType::Player => "player_action",
        ActionType::Entity => "entity_action",
        ActionType::Game => "game_action",
        ActionType::Variable => "set_var",
        ActionType::Control => "control",
        ActionType::Select => "select_obj"
    };

    let mut args: Vec<Arg> = vec![];

    let mut attribute = None;
    let mut sub_action = None;
    let mut target = None;

    if !node.clone().args.is_empty() {
        let arg =  node.args.get(0).clone().unwrap();
        match arg.value.clone() {
            crate::node::ArgValue::Condition { name, args: new_args, selector, inverted, .. } => {
                for arg in new_args {
                    let arg = match arg_val_from_arg(arg, node.name.clone(), "repeat".to_owned()) {
                        Some(res) => res,
                        None => continue
                    };
                    args.push(arg);
                }
                attribute = if inverted {
                    Some("NOT".into())
                } else {
                    None
                };
                sub_action = Some(name);
                target = Some(selector);
            }
            _ => {
                for arg in node.args {
                    let arg = match arg_val_from_arg(arg, node.name.clone(), block.to_owned()) {
                        Some(res) => res,
                        None => continue
                    };
                    args.push(arg);
                }
            }
        }
    }

    Block {
        action: Some(node.name),
        block: Some(block.to_string()),
        id: "block".to_string(),
        target: match node.action_type {
            ActionType::Game => None,
            ActionType::Variable => None,
            ActionType::Control => None,
            ActionType::Select => None,
            _ => if target.is_some() {
                target
            } else {
                Some(node.selector)
            }
        },
        args: Some(Args { items: args }),
        attribute,
        data: None,
        direct: None,
        sub_action,
        bracket_type: None
    }
}

fn arg_val_from_arg(arg: crate::node::Arg, node_name: String, block: String) -> Option<Arg> {
    let arg = match arg.value {
        crate::node::ArgValue::Empty => None,
        crate::node::ArgValue::Text { text } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Simple { name: text }, id: String::from("comp") }, slot: arg.index } )       
        }
        crate::node::ArgValue::Number { number } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Simple { name: number.to_string() }, id: String::from("num") }, slot: arg.index} )
        }
        crate::node::ArgValue::String { string } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Simple { name: string }, id: String::from("txt") }, slot: arg.index } )
        }
        crate::node::ArgValue::Location { x, y, z, pitch, yaw } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Location { is_block: false, loc: Location { x, y, z, pitch, yaw } }, id: String::from("loc") }, slot: arg.index } )
        } 
        crate::node::ArgValue::Vector { x, y, z } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Vector { x, y, z }, id: String::from("vec") }, slot: arg.index } )
        }
        crate::node::ArgValue::Sound { sound, volume, pitch } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Sound { sound, volume, pitch }, id: String::from("snd") }, slot: arg.index } )
        }
        crate::node::ArgValue::Potion { potion, amplifier, duration } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Potion { potion, amplifier, duration }, id: String::from("pot") }, slot: arg.index } )
        }
        crate::node::ArgValue::Tag { tag, value, definition, .. } => {
           Some( Arg { item: ArgItem { data: ArgValueData::Tag {
            action: node_name,
            block,
            option: value,
            tag
           }, id: String::from("bl_tag")}, slot: definition.unwrap().slot as i32})
        }
        crate::node::ArgValue::Variable { name, scope } => {
            Some( Arg { item: ArgItem { data: ArgValueData::Variable { name, scope }, id: String::from("var") }, slot: arg.index } )
        }
        crate::node::ArgValue::GameValue { value, selector, .. } => {
            Some ( Arg { item: ArgItem { data: ArgValueData::GameValue { game_value: value, target: selector }, id: String::from("g_val") }, slot: arg.index })
        }
        crate::node::ArgValue::Condition { .. } => {
            unreachable!();
        }
    };
    arg
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Codeline {
    pub blocks: Vec<Block>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Block {
    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Args>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Selector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename="subAction")]
    pub sub_action: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub direct: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename="type")]
    pub bracket_type: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Args {
    pub items: Vec<Arg>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Arg {
    pub item: ArgItem,
    pub slot: i32
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ArgItem {
    pub data: ArgValueData,
    pub id: String
}

#[derive(Debug)]
pub enum ArgValueData {
    Simple { name: String },
    Id { id: String },
    GameValue {
        game_value: String,
        target: Selector
    },
    Variable { name: String, scope: String },
    Location { is_block: bool, loc: Location },
    Vector { x: f32, y: f32, z: f32 },
    Sound { sound: String, volume: f32, pitch: f32 },
    Potion { potion: String, amplifier: f32, duration: f32 },
    Tag { action: String, block: String, option: String, tag: String },
    FunctionParam {
        default_value: Option<FunctionDefaultItem>,
        name: String,
        optional: bool,
        plural: bool,
        param_type: String
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FunctionDefaultItem {
    pub data: FunctionDefaultItemData,
    pub id: String
}

#[derive(Deserialize, Debug)]
pub enum FunctionDefaultItemData {
    Simple { name: String },
    Id { id: String },
    Location { is_block: bool, loc: Location },
    Vector { x: f32, y: f32, z: f32 },
    Sound { sound: String, volume: f32, pitch: f32 },
    Potion { potion: String, amplifier: f32, duration: f32 },
}

impl Serialize for ArgValueData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ArgValueData::Simple { name } => {
                let mut state = serializer.serialize_struct("MyEnum", 1)?;
                state.serialize_field("name", name)?;
                state.end()
            }
            ArgValueData::Id { id } => {
                let mut state = serializer.serialize_struct("MyEnum", 1)?;
                state.serialize_field("id", id)?;
                state.end()
            }
            ArgValueData::GameValue { target, game_value } => {
                let mut state = serializer.serialize_struct("MyEnum", 1)?;
                state.serialize_field("type", game_value)?;
                state.serialize_field("target", target)?;
                state.end()
            }
            ArgValueData::Variable { name, scope } => {
                let mut state = serializer.serialize_struct("MyEnum", 2)?;
                state.serialize_field("name", name)?;
                state.serialize_field("scope", scope)?;
                state.end()
            }
            ArgValueData::Location { is_block, loc } => {
                let mut state = serializer.serialize_struct("MyEnum", 2)?;
                state.serialize_field("isBlock", is_block)?;
                state.serialize_field("loc", loc)?;
                state.end()
            }
            ArgValueData::Vector { x, y, z } => {
                let mut state = serializer.serialize_struct("MyEnum", 3)?;
                state.serialize_field("x", x)?;
                state.serialize_field("y", y)?;
                state.serialize_field("z", z)?;
                state.end()
            }
            ArgValueData::Sound { sound, volume, pitch } => {
                let mut state = serializer.serialize_struct("MyEnum", 3)?;
                state.serialize_field("sound", sound)?;
                state.serialize_field("vol", volume)?;
                state.serialize_field("pitch", pitch)?;
                state.end()
            }
            ArgValueData::Potion { potion, amplifier, duration } => {
                let mut state = serializer.serialize_struct("MyEnum", 3)?;
                state.serialize_field("pot", potion)?;
                state.serialize_field("amp", amplifier)?;
                state.serialize_field("dur", duration)?;
                state.end()
            }
            ArgValueData::Tag { action, block, option, tag } => {
                let mut state = serializer.serialize_struct("MyEnum", 4)?;
                state.serialize_field("action", action)?;
                state.serialize_field("block", block)?;
                state.serialize_field("option", option)?;
                state.serialize_field("tag", tag)?;
                state.end()
            }
            ArgValueData::FunctionParam { default_value, name, optional, plural, param_type } => {
                let mut state = serializer.serialize_struct("MyEnum", 4)?;
                if default_value.is_some() {
                    state.serialize_field("default_value", default_value)?;
                }
                state.serialize_field("name", name)?;
                state.serialize_field("optional", optional)?;
                state.serialize_field("plural", plural)?;
                state.serialize_field("type", param_type)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ArgValueData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            Name,
            Id,
            Type,
            Target,
            Scope,
            IsBlock,
            Loc,
            X,
            Y,
            Z,
            Sound,
            Vol,
            Pitch,
            Pot,
            Amp,
            Dur,
            Action,
            Block,
            Option,
            Tag,
            DefaultValue,
            Optional,
            Plural,
        }

        struct ArgValueDataVisitor;

        impl<'de> Visitor<'de> for ArgValueDataVisitor {
            type Value = ArgValueData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ArgValueData")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut id = None;
                let mut game_value = None;
                let mut target = None;
                let mut scope = None;
                let mut is_block = None;
                let mut loc = None;
                let mut x = None;
                let mut y = None;
                let mut z = None;
                let mut sound = None;
                let mut volume = None;
                let mut pitch = None;
                let mut potion = None;
                let mut amplifier = None;
                let mut duration = None;
                let mut action = None;
                let mut block = None;
                let mut option = None;
                let mut tag = None;
                let mut default_value = None;
                let mut optional = None;
                let mut plural = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Field::Type => {
                            if game_value.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }
                            game_value = Some(map.next_value()?);
                        }
                        Field::Target => {
                            if target.is_some() {
                                return Err(de::Error::duplicate_field("target"));
                            }
                            target = Some(map.next_value()?);
                        }
                        Field::Scope => {
                            if scope.is_some() {
                                return Err(de::Error::duplicate_field("scope"));
                            }
                            scope = Some(map.next_value()?);
                        }
                        Field::IsBlock => {
                            if is_block.is_some() {
                                return Err(de::Error::duplicate_field("isBlock"));
                            }
                            is_block = Some(map.next_value()?);
                        }
                        Field::Loc => {
                            if loc.is_some() {
                                return Err(de::Error::duplicate_field("loc"));
                            }
                            loc = Some(map.next_value()?);
                        }
                        Field::X => {
                            if x.is_some() {
                                return Err(de::Error::duplicate_field("x"));
                            }
                            x = Some(map.next_value()?);
                        }
                        Field::Y => {
                            if y.is_some() {
                                return Err(de::Error::duplicate_field("y"));
                            }
                            y = Some(map.next_value()?);
                        }
                        Field::Z => {
                            if z.is_some() {
                                return Err(de::Error::duplicate_field("z"));
                            }
                            z = Some(map.next_value()?);
                        }
                        Field::Sound => {
                            if sound.is_some() {
                                return Err(de::Error::duplicate_field("sound"));
                            }
                            sound = Some(map.next_value()?);
                        }
                        Field::Vol => {
                            if volume.is_some() {
                                return Err(de::Error::duplicate_field("vol"));
                            }
                            volume = Some(map.next_value()?);
                        }
                        Field::Pitch => {
                            if pitch.is_some() {
                                return Err(de::Error::duplicate_field("pitch"));
                            }
                            pitch = Some(map.next_value()?);
                        }
                        Field::Pot => {
                            if potion.is_some() {
                                return Err(de::Error::duplicate_field("pot"));
                            }
                            potion = Some(map.next_value()?);
                        }
                        Field::Amp => {
                            if amplifier.is_some() {
                                return Err(de::Error::duplicate_field("amp"));
                            }
                            amplifier = Some(map.next_value()?);
                        }
                        Field::Dur => {
                            if duration.is_some() {
                                return Err(de::Error::duplicate_field("dur"));
                            }
                            duration = Some(map.next_value()?);
                        }
                        Field::Action => {
                            if action.is_some() {
                                return Err(de::Error::duplicate_field("action"));
                            }
                            action = Some(map.next_value()?);
                        }
                        Field::Block => {
                            if block.is_some() {
                                return Err(de::Error::duplicate_field("block"));
                            }
                            block = Some(map.next_value()?);
                        }
                        Field::Option => {
                            if option.is_some() {
                                return Err(de::Error::duplicate_field("option"));
                            }
                            option = Some(map.next_value()?);
                        }
                        Field::Tag => {
                            if tag.is_some() {
                                return Err(de::Error::duplicate_field("tag"));
                            }
                            tag = Some(map.next_value()?);
                        }
                        Field::DefaultValue => {
                            if default_value.is_some() {
                                return Err(de::Error::duplicate_field("default_value"));
                            }
                            default_value = Some(map.next_value()?);
                        }
                        Field::Optional => {
                            if optional.is_some() {
                                return Err(de::Error::duplicate_field("optional"));
                            }
                            optional = Some(map.next_value()?);
                        }
                        Field::Plural => {
                            if plural.is_some() {
                                return Err(de::Error::duplicate_field("plural"));
                            }
                            plural = Some(map.next_value()?);
                        }
                    }
                }

                if let (Some(name), Some(scope)) = (name.clone(), scope) {
                    Ok(ArgValueData::Variable { name, scope })
                } else if let (Some(name), Some(optional), Some(plural), Some(param_type)) = (name.clone(), optional, plural, game_value.clone()) {
                    Ok(ArgValueData::FunctionParam {
                        default_value,
                        name,
                        optional,
                        plural,
                        param_type,
                    })
                } else if let Some(name) = name {
                    Ok(ArgValueData::Simple { name })
                } else if let Some(id) = id {
                    Ok(ArgValueData::Id { id })
                } else if let (Some(game_value), Some(target)) = (game_value, target) {
                    Ok(ArgValueData::GameValue { game_value, target })
                } else if let (Some(is_block), Some(loc)) = (is_block, loc) {
                    Ok(ArgValueData::Location { is_block, loc })
                } else if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                    Ok(ArgValueData::Vector { x, y, z })
                } else if let (Some(sound), Some(volume), Some(pitch)) = (sound, volume, pitch) {
                    Ok(ArgValueData::Sound { sound, volume, pitch })
                } else if let (Some(potion), Some(amplifier), Some(duration)) =
                    (potion, amplifier, duration)
                {
                    Ok(ArgValueData::Potion {
                        potion,
                        amplifier,
                        duration,
                    })
                } else if let (Some(action), Some(block), Some(option), Some(tag)) =
                    (action, block, option, tag)
                {
                    Ok(ArgValueData::Tag {
                        action,
                        block,
                        option,
                        tag,
                    })
                } else {
                    Err(de::Error::missing_field("required field"))
                }
            }
        }

        deserializer.deserialize_struct("ArgValueData", &[], ArgValueDataVisitor)
    }
}

impl Serialize for FunctionDefaultItemData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FunctionDefaultItemData::Simple { name } => {
                let mut state = serializer.serialize_struct("MyEnum", 1)?;
                state.serialize_field("name", name)?;
                state.end()
            }
            FunctionDefaultItemData::Id { id } => {
                let mut state = serializer.serialize_struct("MyEnum", 1)?;
                state.serialize_field("id", id)?;
                state.end()
            }
            FunctionDefaultItemData::Location { is_block, loc } => {
                let mut state = serializer.serialize_struct("MyEnum", 2)?;
                state.serialize_field("isBlock", is_block)?;
                state.serialize_field("loc", loc)?;
                state.end()
            }
            FunctionDefaultItemData::Vector { x, y, z } => {
                let mut state = serializer.serialize_struct("MyEnum", 3)?;
                state.serialize_field("x", x)?;
                state.serialize_field("y", y)?;
                state.serialize_field("z", z)?;
                state.end()
            }
            FunctionDefaultItemData::Sound { sound, volume, pitch } => {
                let mut state = serializer.serialize_struct("MyEnum", 3)?;
                state.serialize_field("sound", sound)?;
                state.serialize_field("vol", volume)?;
                state.serialize_field("pitch", pitch)?;
                state.end()
            }
            FunctionDefaultItemData::Potion { potion, amplifier, duration } => {
                let mut state = serializer.serialize_struct("MyEnum", 3)?;
                state.serialize_field("pot", potion)?;
                state.serialize_field("amp", amplifier)?;
                state.serialize_field("dur", duration)?;
                state.end()
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Location {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub pitch: Option<f32>,
    pub yaw: Option<f32>,
}

impl Serialize for Location {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MyEnum", 5)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("z", &self.y)?;
        state.serialize_field("y", &self.z)?;
        if self.pitch.is_none() {
            state.serialize_field("pitch", &0)?;
        } else {
            state.serialize_field("pitch", &self.pitch.unwrap())?;
        }
        if self.yaw.is_none() {
            state.serialize_field("yaw", &0)?;
        } else {
            state.serialize_field("yaw", &self.yaw.unwrap())?;
        }
        state.end()
    }
}

pub struct CompiledLine {
    pub name: String,
    pub code: String
}