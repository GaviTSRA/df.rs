use super::action_dump::{get_actions, Action, ActionDump};

#[derive(Debug)]
pub struct PlayerActions {
    player_actions: Vec<Action>
}

impl PlayerActions {
    pub fn new(action_dump: &ActionDump) -> PlayerActions {
        let actions = get_actions(action_dump, "PLAYER ACTION");
        PlayerActions {player_actions: actions}
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.player_actions.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.player_actions
    }
}

#[derive(Debug)]
pub struct EntityActions {
    entity_actions: Vec<Action>
}

impl EntityActions {
    pub fn new(action_dump: &ActionDump) -> EntityActions {
        let actions = get_actions(action_dump, "ENTITY ACTION");
        EntityActions { entity_actions: actions }
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.entity_actions.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.entity_actions
    }
}

#[derive(Debug)]
pub struct GameActions {
    game_actions: Vec<Action>
}

impl GameActions {
    pub fn new(action_dump: &ActionDump) -> GameActions {
        let actions = get_actions(action_dump, "GAME ACTION");
        GameActions { game_actions: actions }
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.game_actions.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.game_actions
    }
}

#[derive(Debug)]
pub struct VariableActions {
    variable_actions: Vec<Action>
}

impl VariableActions {
    pub fn new(action_dump: &ActionDump) -> VariableActions {
        let actions = get_actions(action_dump, "SET VARIABLE");
        VariableActions { variable_actions: actions }
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.variable_actions.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.variable_actions
    }
}


#[derive(Debug)]
pub struct ControlActions {
    control_actions: Vec<Action>
}

impl ControlActions {
    pub fn new(action_dump: &ActionDump) -> ControlActions {
        let actions = get_actions(action_dump, "CONTROL");
        ControlActions { control_actions: actions }
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.control_actions.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.control_actions
    }
}

#[derive(Debug)]
pub struct SelectActions {
    select_actions: Vec<Action>
}

impl SelectActions {
    pub fn new(action_dump: &ActionDump) -> SelectActions {
        let actions = get_actions(action_dump, "SELECT OBJECT");
        SelectActions {select_actions: actions}
    }

    pub fn get(&self, dfrs_name: String) -> Option<&Action> {
        self.select_actions.iter().find(|&action| action.dfrs_name == dfrs_name)
    }

    pub fn all(&self) -> &Vec<Action> {
        &self.select_actions
    }
}

pub fn get_start_process_action(action_dump: &ActionDump) -> Action {
    let actions  = get_actions(&action_dump, "START PROCESS");
    let action = actions.get(0).unwrap();
    Action {
        args: action.args.clone(),
        df_name: action.df_name.clone(),
        dfrs_name: action.dfrs_name.clone(),
        tags: action.tags.clone(),
        has_conditional_arg: action.has_conditional_arg.clone()
    }
}