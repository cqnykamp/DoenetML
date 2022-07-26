pub mod prelude;

pub mod state_variables;
pub mod component;

pub mod state_var;
pub mod parse_json;
pub mod utils;

use std::collections::HashMap;
use std::fmt::Debug;

use crate::prelude::*;
use crate::component::*;

use state_var::EssentialStateVar;
use state_variables::*;
use state_var::State;


/// This stores the components and dependency graph.
#[derive(Debug)]
pub struct DoenetCore {
    pub component_nodes: HashMap<String, ComponentNode>,
    pub component_states: HashMap<String, Box<dyn ComponentStateVars>>,
    pub dependencies: HashMap<String, HashMap<StateVarName, Vec<Dependency>>>,
    pub root_component_name: String,

    /// We send components to the renderer that do not exist: the inherited children of a copy.
    /// The renderer needs to recognize these as a different component so we alias its name.
    /// This maps the renderer's to the actual name.
    pub aliases: HashMap<String, String>,
}


/// This stores some of the state variables (or strings) that a state variable depends on.
/// Note that a Dependency struct contains multiple edges of the dependency tree.
#[derive(Debug)]
pub struct Dependency {

    pub component: String,
    pub state_var: StateVarName,

    pub name: InstructionName,


    // We will use outer product of entries (except for the strings, which don't have state vars)
    pub depends_on_objects: Vec<ObjectName>,
    pub depends_on_state_vars: Vec<StateVarName>,

    // TODO: Do we really need this field? It would be easier if we didn't
    // pub instruction: DependencyInstruction,


    pub variables_optional: bool,
}




pub fn create_doenet_core(program: &str) -> DoenetCore {

    // Create component nodes.
    let (component_nodes, root_component_name) = parse_json::create_components_tree_from_json(program)
        .expect(&format!("Error parsing json for components: {}", program));

    log!("Component nodes from JSON {:#?}", component_nodes);


    // For every copy, add aliases for children it would inherit from its copy target.
    let mut aliases: HashMap<String, String> = HashMap::new();
    let copies = component_nodes.iter().filter(|(_, c)| c.copy_target.is_some());
    for (_name, copy) in copies {
        add_alias_for_children(&mut aliases, copy, &component_nodes, &copy.name);
    }

    // Fill in component states and dependencies for every component.
    let mut component_states: HashMap<String, Box<dyn ComponentStateVars>> = HashMap::new();

    // let mut dependencies: Vec<Dependency> = vec![];
    let mut dependencies:
        HashMap<String,
            HashMap<StateVarName, Vec<Dependency>>> = HashMap::new();
    
    for (component_name, component_node) in component_nodes.iter() {

        let dependencies_for_this_component = create_all_dependencies_for_component(&component_nodes, component_node);

        dependencies.insert(component_name.to_string(), dependencies_for_this_component);



        component_states.insert(component_name.clone(),

            // Only give this component essential data if it is not copying anything
            component_node.definition.new_stale_component_state_vars(component_node.copy_target.is_none())
        );

    }


    // log!("component nodes {:#?}", component_nodes);
    // log!("component states {:#?}", component_states);

    DoenetCore {
        component_nodes,
        component_states,
        dependencies,
        root_component_name,
        aliases,
    }
}


fn name_child_of_copy(child: &str, copy: &str) -> String {
    format!("__cp:{}({})", child, copy)
}

fn add_alias_for_children(
    aliases: &mut HashMap<String, String>,
    component: &ComponentNode,
    component_nodes: &HashMap<String, ComponentNode>,
    copy: &String,
) {
    for (child, _) in get_children_including_copy(component_nodes, component).iter() {
        if let ComponentChild::Component(child_comp) = child {
            aliases.insert(name_child_of_copy(child_comp, copy), child_comp.to_string());
        }
    }
}


fn create_all_dependencies_for_component(
    components: &HashMap<String, ComponentNode>,
    component: &ComponentNode,
) -> HashMap<StateVarName, Vec<Dependency>>

{

    log!("Creating depencies for {:?}", component.name);
    let mut dependencies: HashMap<StateVarName, Vec<Dependency>> = HashMap::new();


    let my_definitions = component.definition.state_var_definitions();
    for (&state_var_name, state_var_def) in my_definitions {

        let dependency_instructions_hashmap = state_var_def.return_dependency_instructions(HashMap::new());
        for (dep_name, ref dep_instruction) in dependency_instructions_hashmap.into_iter() {

            let dependency =  create_dependency_from_instruction(&components, component, state_var_name, dep_instruction, dep_name);

            dependencies.entry(state_var_name).or_insert(vec![]).push(dependency);


        }
    }

    dependencies


    // component.definition.state_var_definitions()
    //     .iter()
    //     .flat_map(|(&sv_name, sv_def)|
    //         sv_def.return_dependency_instructions(HashMap::new())
    //             .iter()
    //             .map(|(dep_name, dep_instruction)|
    //                 create_dependency_from_instruction(&components, component, sv_name, dep_instruction, dep_name)
    //             )
    //             .collect::<Vec<_>>()
    //     )
    //     .collect()


    //     }




    // }

    // dependencies

}

/// Get the specified if it exists on this component, or on the component it copies
/// Recursively searches the target (and the target's target if it has one), and finds
/// the nearest attribute to the original node, if any exist
fn get_attribute_including_copy<'a>(
    components: &'a HashMap<String, ComponentNode>,
    component: &'a ComponentNode,
    attribute_name: AttributeName
)-> Option<&'a Attribute> {

    // let target = components.get(target_name).unwrap();
    if let Some(attribute) = component.attributes.get(attribute_name) {
        Some(attribute)

    } else if let Some(ref target_name) = component.copy_target {

        let target = components.get(target_name).unwrap();
        get_attribute_including_copy(components, target, attribute_name)

    } else {
        None
    }
}


fn create_dependency_from_instruction(
    components: &HashMap<String, ComponentNode>,
    component: &ComponentNode,
    state_var_name: StateVarName,
    instruction: &DependencyInstruction,
    instruction_name: InstructionName,

) -> Dependency {

    let depends_on_objects: Vec<ObjectName>;
    let depends_on_state_vars: Vec<StateVarName>;

    log!("Creating dependency {}:{}:{}", component.name, state_var_name, instruction_name);


    match &instruction {

        DependencyInstruction::StateVar(state_var_instruction) => {

            depends_on_objects = if let Option::Some(ref name) = state_var_instruction.component_name {
                    vec![ObjectName::Component(name.to_string())]
                } else {
                    vec![ObjectName::Component(component.name.clone())]
                };
            depends_on_state_vars = vec![state_var_instruction.state_var];
        },

        DependencyInstruction::Child(child_instruction) => {

            let mut depends_on_children: Vec<ObjectName> = vec![];
            for (child, _) in get_children_including_copy(components, component).iter() {

                for desired_child_type in child_instruction.desired_children.iter() {
                    match child {
                        ComponentChild::Component(child_component_name) => {
                            let child_component = components.get(child_component_name).unwrap();

                            if child_component.definition.get_trait_names().contains(desired_child_type) {
                                // If not already in list, add it to the list
                                if !depends_on_children.contains(&ObjectName::Component(child_component.name.clone())) {
                                    depends_on_children.push(ObjectName::Component(child_component.name.clone()));
                                }
                            }
                        },

                        ComponentChild::String(string_value) => {
                            if desired_child_type == &ObjectTraitName::TextLike ||
                                desired_child_type == &ObjectTraitName::NumberLike {
                                
                                depends_on_children.push(ObjectName::String(string_value.to_owned()));

                            }
                        },
                    }

                }
            }

            depends_on_objects = depends_on_children;
            depends_on_state_vars = child_instruction.desired_state_vars.clone();

        },
        DependencyInstruction::Parent(parent_instruction) => {
            // Parent doesn't exist yet

            let parent_name = component.parent.clone().expect(&format!(
                "Component {} doesn't have a parent, but the dependency instruction {}:{} asks for one.",component.name, state_var_name, instruction_name
            ));

            depends_on_objects = vec![ObjectName::Component(parent_name)];
            depends_on_state_vars = vec![parent_instruction.state_var];
        },


        DependencyInstruction::Attribute(attribute_instruction) => {

            log!("attribute instruction {:#?}", attribute_instruction);
            // log!("component attributes {:#?}", component.attributes());

            let attribute_name = attribute_instruction.attribute_name;

            let possible_attribute: Option<&Attribute> = get_attribute_including_copy(components, component, attribute_name);


            if let Some(attribute) = possible_attribute {
                match attribute {
                    Attribute::Component(attr_comp_name) => {
                        depends_on_objects = vec![ObjectName::Component(attr_comp_name.to_string())];

                        // hard code this for now
                        depends_on_state_vars = vec!["value"];
                    },

                    Attribute::Primitive(attr_primitive_value) => {
                        depends_on_objects = vec![ObjectName::String(

                            // for now, convert it to a string
                            match attr_primitive_value {
                                StateVarValue::String(v) => v.to_string(),
                                StateVarValue::Boolean(v) => v.to_string(),
                                StateVarValue::Number(v) => v.to_string(),
                                StateVarValue::Integer(v) => v.to_string(),
                            }
                        )];

                        depends_on_state_vars = vec![];
                    }
                }

            } else {
                // Attribute doesn't exist
                depends_on_objects = vec![];
                depends_on_state_vars = vec![];
            }

        }
    };


    Dependency {
        name: instruction_name,
        component: component.name.clone(),
        state_var: state_var_name,
        variables_optional: false,

        depends_on_objects,
        depends_on_state_vars,
    }


}




fn resolve_state_variable(
    core: &DoenetCore,
    component: &ComponentNode,
    state_var_name: StateVarName
) -> StateVarValue {
        
    let state_vars = core.component_states.get(&component.name).unwrap();

    let mut dependency_values: HashMap<InstructionName, Vec<DependencyValue>> = HashMap::new();

    // log!("Resolving {}:{}", component.name, state_var_name);

    let empty_vec = vec![];
    let my_dependencies = core.dependencies.get(&component.name).unwrap().get(state_var_name).unwrap_or(&empty_vec);

    for dep in my_dependencies {

        let mut values_for_this_dep: Vec<DependencyValue> = Vec::new();

        for depends_on in &dep.depends_on_objects {

            match depends_on {
                ObjectName::String(string) => {

                    // Right now, the only thing you can get from a string is its faked 'value' state var
                    if dep.depends_on_state_vars.contains(&"value") {
                        values_for_this_dep.push(DependencyValue {
                            component_type: "string",
                            state_var_name: "value",
                            value: StateVarValue::String(string.to_string()),
                        });
               
                    }
                },
                ObjectName::Component(depends_on_name) => {

                    let depends_on_component = core.component_nodes.get(depends_on_name).unwrap();

                    for &dep_state_var_name in &dep.depends_on_state_vars {

                        // log!("About to recurse and resolve {}:{}", depends_on_component.name(), dep_state_var_name);

                        let depends_on_value = resolve_state_variable(core, depends_on_component, dep_state_var_name);

                        values_for_this_dep.push(DependencyValue {
                            component_type: core.component_nodes.get(depends_on_name).unwrap().component_type,
                            state_var_name: dep_state_var_name,
                            value: depends_on_value.clone(),
                        });

                    }
                }
            }
        }

        // log!("dep name {}", dep.name);
        dependency_values.insert(dep.name, values_for_this_dep);
    }

    let definition = component.definition.state_var_definitions().get(state_var_name).unwrap();

    let update_instruction = definition.determine_state_var_from_dependencies(dependency_values);
    let new_value = handle_update_instruction(core, component, state_vars.as_ref(), state_var_name, update_instruction);

    return new_value;

}




fn mark_stale_state_var_and_dependencies(
    core: &DoenetCore,
    component: &ComponentNode,
    state_var_name: StateVarName)
{

    // log!("Marking stale {}:{}", component.name(), state_var_name);

    let component_state = core.component_states.get(&component.name).unwrap();

    let state_var = component_state.get(state_var_name).unwrap();
    state_var.mark_stale();

    let my_dependencies = core.dependencies.get(&component.name).unwrap().get(state_var_name).unwrap();
    
    for dependency in my_dependencies {

        for depends_on in &dependency.depends_on_objects {
            match depends_on {
                ObjectName::String(_) => {
                    // do nothing
                },
                ObjectName::Component(dep_comp_name) => {
                    let dep_component = core.component_nodes.get(dep_comp_name).unwrap();

                    for &dep_state_var_name in &dependency.depends_on_state_vars {

                        mark_stale_state_var_and_dependencies(core, dep_component, dep_state_var_name);
                    }
                }
            }
        }
    }

}


/// Sets the state var and returns the new value
fn handle_update_instruction<'a>(
    core: &DoenetCore,
    component: &'a ComponentNode,
    component_state_vars: &dyn ComponentStateVars,
    name: StateVarName,
    instruction: StateVarUpdateInstruction<StateVarValue>
) -> StateVarValue {

    log!("Updating {}:{}", component.name, name);

    let definition = component.definition.state_var_definitions().get(name).unwrap();

    let updated_value: StateVarValue;

    match instruction {
        StateVarUpdateInstruction::NoChange => {
            let current_value = component_state_vars.get(name).unwrap().get_state();

            if let State::Resolved(current_resolved_value) = current_value {
                // Do nothing. It's resolved, so we can use it as is
                updated_value = current_resolved_value;

            } else {
                panic!("Cannot use NoChange update instruction on a stale value");
            }

        },
        StateVarUpdateInstruction::UseEssentialOrDefault => {
            if definition.has_essential() == false {
                panic!(
                    "Cannot UseEssentialOrDefault on {}:{},
                    which has no essential (Component type {}) ",
                    component.name, name, component.component_type
                );
            }

            let (essential_state_var, _) = get_essential_var_considering_copy(&core.component_nodes, &core.component_states, component, name);

            let possible_essential_val = essential_state_var.get_value();
            let new_state_var_value = if let Some(actual_val) = possible_essential_val {
                actual_val
            } else {
                definition.default_value()
            };
            

            set_state_var(component, component_state_vars, name, new_state_var_value.clone()).expect(
                &format!("Failed to set {}:{}", component.name, name)
            );
            updated_value = new_state_var_value;

        },
        StateVarUpdateInstruction::SetValue(new_value) => {

            let new_state_var_value = new_value;
            set_state_var(component, component_state_vars, name, new_state_var_value.clone()).expect(
                &format!("Failed to set {}:{}", component.name, name)
            );

            updated_value = new_state_var_value;
        }

    };


    return updated_value;
}


fn set_state_var(
    component: &ComponentNode,
    component_state_vars: &dyn ComponentStateVars,
    name: StateVarName,
    val: StateVarValue)
-> Result<(), String> {
    let state_var = component_state_vars.get(name).expect(
        &format!("Component {} of type {} does not have state var {}",
        component.name, component.component_type, name)
    );

    state_var.set_value(val)
}

/// Get the essential state var that this component owns or copies from
/// Also returns the name of the owning component
fn get_essential_var_considering_copy<'a>(
    components: &'a HashMap<String, ComponentNode>,
    component_states: &'a HashMap<String, Box<dyn ComponentStateVars>>,
    component: &'a ComponentNode,
    essential_var_name: &str,
) -> (&'a EssentialStateVar, &'a str) {

    // If this is a copyTarget component, use the target's essen var
    if let Some(ref target_name) = component.copy_target {
        let target = components.get(target_name).unwrap();

        get_essential_var_considering_copy(components, component_states, target, essential_var_name)

    } else {
        let component_state = component_states.get(&component.name).unwrap();

        let essential_var = component_state.get_essential_state_vars().get(essential_var_name).unwrap();

        (essential_var, &component.name)
    }
}





#[derive(Debug)]
pub struct Action {
    pub component_name: String,
    pub action_name: String,
    pub args: HashMap<String, StateVarValue>,
}


pub fn handle_action_from_json(core: &DoenetCore, action: &str) {
    
    let action = parse_json::parse_action_from_json(action)
        .expect(&format!("Error parsing json action: {}", action));

    // log!("Handling action {:#?}", action);

    // Apply alias to get the original component name
    let component_name = core.aliases.get(&action.component_name).unwrap_or(&action.component_name);

    let component = core.component_nodes.get(component_name)
        .expect(&format!("{} doesn't exist, but action {} uses it", action.component_name, action.action_name));

    let state_var_resolver = | state_var_name | {
        resolve_state_variable(core, component, state_var_name)
    };

    let state_vars_to_update = component.definition.on_action(&action.action_name, action.args, &state_var_resolver);

    for (name, requested_value) in state_vars_to_update {

        let definition = component.definition.state_var_definitions().get(name).unwrap();
        let requests = definition.request_dependencies_to_update_value(requested_value);

        for request in requests {
            process_update_request(core, component, name, &request);

        }
    }

}


fn process_update_request(
    core: &DoenetCore,
    component: &ComponentNode,
    state_var_name: StateVarName,
    update_request: &UpdateRequest) 
{

    // log!("Processing update request for {}:{}", component.name(), state_var_name);

    let component_to_mark_stale: &ComponentNode;

    match update_request {
        UpdateRequest::SetEssentialValue(their_name, requested_value) => {

            let (essential_var, owning_comp_name) = get_essential_var_considering_copy(&core.component_nodes, &core.component_states, component, their_name);

            essential_var.set_value(requested_value.clone()).expect(
                &format!("Failed to set essential value for {}:{}", component.name, their_name)
            );

            let owning_component = core.component_nodes.get(owning_comp_name).unwrap();


            component_to_mark_stale = owning_component;
        },

        UpdateRequest::SetStateVarDependingOnMe(their_name, requested_value) => {

            log!("desired value {:?}", requested_value);


            let state_var_definition = component.definition.state_var_definitions().get(their_name).unwrap();

            let their_update_requests = state_var_definition.request_dependencies_to_update_value(requested_value.clone());

            for their_update_request in their_update_requests {
                process_update_request(core, component, their_name, &their_update_request);
            }

            component_to_mark_stale = component;
        }
    }

    mark_stale_state_var_and_dependencies(core, component_to_mark_stale, state_var_name);


}




pub fn update_renderers(core: &DoenetCore) -> String {
    let json_obj = generate_render_tree(core);
    serde_json::to_string(&json_obj).unwrap()
}


fn generate_render_tree(core: &DoenetCore) -> serde_json::Value {

    let root_node = core.component_nodes.get(&core.root_component_name).unwrap();
    let mut json_obj: Vec<serde_json::Value> = vec![];

    generate_render_tree_internal(core, root_node, &mut json_obj, None);

    serde_json::Value::Array(json_obj)
}

fn generate_render_tree_internal(
    core: &DoenetCore,
    component: &ComponentNode,
    json_obj: &mut Vec<serde_json::Value>,
    came_from_copy: Option<&String>,
) {

    let component_state = core.component_states.get(&component.name).unwrap();

    let comp_state_vars = component_state.as_ref();

    use serde_json::json;

    let state_vars = component.definition.state_var_definitions();

    let renderered_state_vars = state_vars.into_iter().filter(|kv| match kv.1 {
        StateVarVariant::Integer(sv) => sv.for_renderer,
        StateVarVariant::Number(sv) =>  sv.for_renderer,
        StateVarVariant::String(sv) =>  sv.for_renderer,
        StateVarVariant::Boolean(sv) => sv.for_renderer,
    });

    let mut state_values = serde_json::Map::new();
    for (name, _variant) in renderered_state_vars {

        resolve_state_variable(core, component, name);

        let state_var_value = comp_state_vars.get(name).unwrap()
            .copy_value_if_resolved().unwrap();

        state_values.insert(name.to_string(), match state_var_value {
            StateVarValue::Integer(v) => json!(v),
            StateVarValue::Number(v) =>  json!(v),
            StateVarValue::String(v) =>  json!(v),
            StateVarValue::Boolean(v) => json!(v),
        });
    }

    let name_to_render = match &came_from_copy {
        Some(copy_name) => name_child_of_copy(&component.name, &copy_name),
        None => component.name.clone(),
    };

    log!("component children {:#?}", component.children);

    let mut children_instructions = Vec::new();
    if component.definition.should_render_children() {
        for (child, actual_child) in get_children_including_copy(&core.component_nodes, component).iter() {
            match child {
                ComponentChild::Component(comp_name) => {
                    // recurse for children
                    let comp = core.component_nodes.get(comp_name).unwrap();
                    
                    let child_came_from_copy =
                        came_from_copy.or(
                            if *actual_child {
                                None
                            } else {
                                Some(&component.name)
                            });

                    generate_render_tree_internal(core, comp, json_obj, child_came_from_copy); 

                    let mut child_actions = serde_json::Map::new();

                    for action_name in comp.definition.action_names() {
                        child_actions.insert(action_name.to_string(), json!({
                            "actionName": action_name,
                            "componentName": comp.name,
                        }));
                    }

                    children_instructions.push(json!({
                        "actions": child_actions,
                        "componentName": comp.name,
                        "componentType": comp.component_type,
                        "effectiveName": comp.name,
                        "rendererType": comp.component_type,
                    }));
                },
                ComponentChild::String(string) => {
                    children_instructions.push(json!(string));
                },
            }
        }
    }

    json_obj.push(json!({
        "componentName": name_to_render,
        "stateValues": serde_json::Value::Object(state_values),
        "childrenInstructions": json!(children_instructions),
    }));

}

/// This includes the copy target's children. The flag is false when it is
/// a copy target's child.
fn get_children_including_copy(
    components: &HashMap<String, ComponentNode>,
    component: &ComponentNode
) -> Vec<(ComponentChild, bool)> {
    let mut children_vec: Vec<(ComponentChild, bool)> = Vec::new();
    if let Some(ref target) = component.copy_target {
        let target_comp = components.get(target).unwrap();
        children_vec = get_children_including_copy(components, target_comp)
            .iter()
            .map(|(c, _)| (c.clone(), false))
            .collect();
    }
    children_vec.extend(
        component.children
            .iter()
            .map(|c| (c.clone(), true))
        );
    children_vec
}



/// List components and children in a JSON array
pub fn json_components(core: &DoenetCore) -> serde_json::Value {

    let json_components: serde_json::Map<String, serde_json::Value> = core.component_nodes
        .values()
        .map(|component| (component.name.to_string(),
                utils::package_subtree_as_json(
                    &core.component_nodes,
                    &&core.component_states,
                    component)))
        .collect();

    serde_json::Value::Object(json_components)
}
