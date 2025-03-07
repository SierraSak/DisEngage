#![feature(ptr_sub_ptr)]

mod enume;
use enume::ReUniteTargetEnumerator;

use std::sync::OnceLock;

use mapunitcommand::{MapUnitCommandMenu, TradeMenuItem};
use unity::{ prelude::*, system::List };

use engage::{
    force::ForceType, force::Force, gamedata::{person, skill::SkillData, unit::Unit, unit::GodUnit, Gamedata, PersonData, GodData}, gamesound::GameSound, mapmind::MapMind, menu::*, proc::{desc::ProcDesc, Bindable, ProcInst, ProcInstFields}, sequence::{
        mapsequence::human::MapSequenceHuman,
        mapsequencetargetselect::{MapSequenceTargetSelect, MapTarget}
    }, titlebar::TitleBar, util::{get_instance, get_singleton_proc_instance}
};

#[unity::class("", "MapUnitCommandMenu.SkillAttackMenuItem")]
pub struct SkillAttackMenuItem {
    base: BasicMenuItemFields,
    command_skill: Option<&'static SkillData>
}

#[unity::class("", "MapUnitCommandMenu.EngageSummonMenuItem")]
pub struct EngageSummonMenuItem {
    base: SkillAttackMenuItem
}

#[unity::class("App", "MapSequenceMind")]
pub struct MapSequenceMind {
    pub proc: ProcInstFields,
    pub unit: Option<&'static Unit>,
    pub target: Option<&'static Unit>,
}

impl Bindable for MapSequenceMind { }

impl AsRef<ProcInstFields> for MapSequenceMind {
    fn as_ref(&self) -> &ProcInstFields {
        &self.proc
    }
}

impl AsMut<ProcInstFields> for MapSequenceMind {
    fn as_mut(&mut self) -> &mut ProcInstFields {
        &mut self.proc
    }
}

#[unity::class("App", "MapSequenceEngageSummon")]
pub struct MapSequenceEngageSummon {
    pub proc: ProcInstFields,
    pub person: &'static mut PersonData,
    pub rank: i32,
}

impl Bindable for MapSequenceEngageSummon { }

impl AsRef<ProcInstFields> for MapSequenceEngageSummon {
    fn as_ref(&self) -> &ProcInstFields {
        &self.proc
    }
}

impl AsMut<ProcInstFields> for MapSequenceEngageSummon {
    fn as_mut(&mut self) -> &mut ProcInstFields {
        &mut self.proc
    }
}

#[unity::class("App", "InfoUtil")]
pub struct InfoUtil { }

impl InfoUtil {
    pub fn try_set_text(tmp: &(), string: impl Into<&'static Il2CppString>) {
        unsafe { infoutil_trysettext(tmp, string.into(), None) }
    }
}

pub struct MapBattleInfoParamSetter { }

impl MapBattleInfoParamSetter {

    pub fn set_battle_info_for_no_param(&self, isweapon: bool, isgodname: bool) {
        unsafe { mapbattleinfoparamsetter_setbattleinfofornoparam(self, isweapon, isgodname, None) }
    }
}

#[unity::class("App", "MapBattleInfoRoot")]
pub struct MapBattleInfoRoot {
    sup: [u8;0x10],
    command_root: &'static (),
    command_sub_root: &'static (),
    command_text: &'static (),
    command_sub_text: &'static (),
    info_left: &'static (),
    info_right: &'static (),
}

#[unity::from_offset("App", "MapBattleInfoParamSetter", "SetBattleInfoForNoParam")]
fn mapbattleinfoparamsetter_setbattleinfofornoparam(this: &MapBattleInfoParamSetter, isweapon: bool, isgodname: bool, method_info: OptionalMethod);

#[unity::from_offset("App", "InfoUtil", "TrySetText")]
fn infoutil_trysettext(tmp: &(), str: &'static Il2CppString, method_info: OptionalMethod);

#[unity::from_offset("App", "MapTarget", "EnumerateSelfOnly")]
fn maptarget_enumerateselfonly(this: &mut MapTarget, itemmask: i32, _method_info: OptionalMethod);

#[skyline::from_offset(0x2d533d0)]
fn fade_isfadeout(_method_info: OptionalMethod) -> bool;

#[skyline::from_offset(0x01f262a0)]
fn persondata_getsummongod(this: &PersonData, _method_info: OptionalMethod) -> Option<&'static Il2CppString>;

#[unity::from_offset("App", "UnitUtil", "SummonDeleteImpl")]
fn unitutil_summondeleteimpl(unit: &Unit, _method_info: OptionalMethod);

#[skyline::from_offset(0x02334570)]
fn godpool_tryget(gid: &'static Il2CppString, _method_info: OptionalMethod) -> Option<&'static GodUnit>;

#[unity::from_offset("App", "UnitPool", "GetForce")]
fn unitpool_get_force(index: i32, _method_info: OptionalMethod) -> Option<&'static Force>;

static DISENGAGE_CLASS: OnceLock<&'static mut Il2CppClass> = OnceLock::new();

//Delete separated Emblems from the player force on map end.
#[unity::hook("App", "MapSequence", "Complete")]
pub fn mapsequence_complete(this: &mut (), _method_info: OptionalMethod) {
    call_original!(this, _method_info);
    let force = unsafe{unitpool_get_force(0, _method_info)};
    let mut unit = force.unwrap().head;

    loop {
        if unit.is_some() {
            if unit.unwrap().get_pid().contains("PID_SUMMON") {
                unsafe{unitutil_summondeleteimpl(unit.unwrap(), _method_info);}
            }
            unit = unit.unwrap().next;
        }
        else {
            return;
        }
    }
    return;
}

// This function is what sets the text that appears in between the two windows
// when targetting another unit.
#[unity::hook("App", "MapBattleInfoRoot", "SetCommandText")]
pub fn mapbattleinforoot_setcommandtext(this: &mut MapBattleInfoRoot, mind_type: i32, _method_info: OptionalMethod) {
    if mind_type != 0x39 {
        call_original!(this, mind_type, _method_info);
    } else {
        let maptarget_instance = get_instance::<MapTarget>();
        if maptarget_instance.unit.unwrap().get_god_unit().is_some() {
            InfoUtil::try_set_text(&this.command_text, "Separate");
        }
        else {
            InfoUtil::try_set_text(&this.command_text, "Re-Unite");
        }
        
    }
}

// Makes the game hide the damage forecast arrows.
// This function is primarily for setting the
// command name in between the two windows, and deciding whether to hide the damage arrows.
// Thankfully, the default behavior is almost exactly what we want, we just need to adjust it
// to return false, since that's what hides the damage arrows.
#[unity::hook("App", "MapBattleInfoRoot", "Setup")]
pub fn mapbattleinforoot_setup(this: &(), mindtype: i32, skill: &SkillData, info: &(), scene_list: &(), _method_info: OptionalMethod) -> bool {
  
    let mut result = call_original!(this, mindtype, skill, info, scene_list, _method_info);

    if mindtype == 0x39 {
        result = false;
    }

    result
}


// This function is responsible for the windows that pop up when you highlight a target.
// The default behavior without this hook makes the battle forecast appear.  So weapons, hp, etc.
#[unity::hook("App", "MapBattleInfoParamSetter", "SetBattleInfo")]
pub fn mapbattleinfoparamsetter_setbattleinfo(this: &mut MapBattleInfoParamSetter, side_type: i32, show_window: bool, battle_info: &(), scene_list: &(), _method_info: OptionalMethod) {
    call_original!(this, side_type, show_window, battle_info, scene_list, _method_info);

    let maptarget_instance = get_instance::<MapTarget>();

    let cur_mind = maptarget_instance.m_mind;

    if cur_mind == 0x39 && maptarget_instance.unit.unwrap().get_god_unit().is_none() {
        this.set_battle_info_for_no_param(false, false);
    }
}


// This is a generic function that essentially checks the Mind value, and then calls
// a more specialized Enumerate function based on the result.
// Enumerate functions are used for checking if there is a valid target in range,
// and making a list of them.
#[unity::hook("App", "MapTarget", "Enumerate")]
pub fn maptarget_enumerate(this: &mut MapTarget, mask: i32, _method_info: OptionalMethod) {
    
    if this.m_mind < 0x38 {
        call_original!(this, mask, _method_info);
    }
    if this.m_mind == 0x39 {
        this.m_action_mask = mask as u32;

        if let Some(unit) = this.unit {
            if this.x < 0 {
                this.x = unit.x as i8;
            }

            if this.z < 0 {
                this.z = unit.z as i8;
            }
        }

        if let Some(dataset) = this.m_dataset.as_mut() {
            dataset.clear();
        }
        if this.unit.unwrap().get_god_unit().is_some() {
            unsafe {maptarget_enumerateselfonly(this, 0, None)};
        }
        else {
            this.enumerate_reunite();
        }

        if let Some(dataset) = this.m_dataset.as_mut() {
            dataset.m_list
                .iter_mut()
                .enumerate()
                .for_each(|(count_var, data_item)| {
                    data_item.m_index = count_var as i8;    
                });
        }
    }
}


// This is the function that usually runs when you press A while highlighting a target and the
// forecast windows are up.
#[unity::hook("App", "MapSequenceTargetSelect", "DecideNormal")]
pub fn mapsequencetargetselect_decide_normal(this: &mut MapSequenceTargetSelect, _method_info: OptionalMethod) {
    let maptarget_instance = get_instance::<MapTarget>();


    let mut cur_mind = maptarget_instance.m_mind;

    if cur_mind == 0x39 {
        let mapsequencehuman_instance = get_singleton_proc_instance::<MapSequenceHuman>().unwrap();

        if let Some(unit) = maptarget_instance.unit {
            if unit.get_god_unit().is_none() {
                if let Some(target) = this.target_data {
                    let person_data = PersonData::get_list_mut().expect("Couldn't reach PersonData List")
                        .iter()
                        .find(|curr_char|curr_char.pid == target.m_unit.get_pid());

                    if let Some(person) = person_data {
                        println!("Person: {}", person.get_ascii_name().unwrap());

                        let god_data = GodData::get_list_mut().expect("Couldn't reach GodData List")
                            .iter()
                            .find(|curr_god| {
                                curr_god.get_gid().unwrap().to_string() == ("GID_".to_owned() + &unsafe{ persondata_getsummongod(person, _method_info) }.unwrap().to_string())
                            });
                        
                        if let Some(god) = god_data {
                            println!("God: {}", god.ascii_name.unwrap());

                            if let Some(god_unit) = unsafe { godpool_tryget(god.get_gid().unwrap(), _method_info) } {
                                println!("God: {}", god_unit.data.ascii_name.unwrap());

                                maptarget_instance.unit.unwrap().set_god_unit(god_unit);
                                maptarget_instance.unit.unwrap().update();

                                unsafe { unitutil_summondeleteimpl(target.m_unit, _method_info); }
                            }
                        }
                    }

                    let mapmind_instance = get_instance::<MapMind>();

                    mapmind_instance.mind = 1;
                    maptarget_instance.m_mind = 1;
                }
            } else {
                this.set_mapmind();
            }
        }

        // Jump to MapSequenceHuman::Mind label
        ProcInst::jump(mapsequencehuman_instance, 0x2E);

        GameSound::post_event("Decide", None);
    } else {
        call_original!(this, _method_info)
    }
}

//This is the function that builds the summon menu allowing you to pick your desired orb color.
#[unity::hook("App", "MapSummonMenu", "CreateSummonBind")]
pub fn mapsummonmenu_createsummonbind(sup: &mut ProcInst, _method_info: OptionalMethod) {
    let maptarget_instance = get_instance::<MapTarget>();
    let cur_mind = maptarget_instance.m_mind;
    if cur_mind == 0x39 {
        let mapmind_instance = get_instance::<MapMind>();
        mapmind_instance.item_index = 1;
        mapmind_instance.x = maptarget_instance.unit.unwrap().x as i8;
        mapmind_instance.z = maptarget_instance.unit.unwrap().z as i8;
        ProcInst::jump(get_singleton_proc_instance::<MapSequenceHuman>().unwrap(),0x20);
    }
    else {
        call_original!(sup, _method_info);
    }
}

//This function reads your mind value, and proc::jumps to the desired proc location.
#[unity::hook("App", "MapSequenceMind", "Branch")]
pub fn mapsequencemind_branch(this: &mut MapSequenceMind, _method_info: OptionalMethod) {
    
    call_original!(this, _method_info);
    let mapmind_instance = get_instance::<MapMind>();
    let cur_mind = mapmind_instance.mind;

    if cur_mind == 0x39 {
        ProcInst::jump(this, 0x16);
    }
}

//This function determines if you have combat anims on, and then proc::jumps
//to the appropriate part of MapSequenceEngageSummon's procs based on that.
#[unity::hook("App", "MapSequenceEngageSummon", "Branch")]
pub fn mapsequenceengagesummon_branch(this: &mut MapSequenceEngageSummon, _method_info: OptionalMethod) {
    let mapmind_instance = get_instance::<MapMind>();
    if mapmind_instance.mind == 0x39 {

        let mut jump_label = 0;

        if unsafe{fade_isfadeout(_method_info)} {
            jump_label = 2;
        }
        else {
            jump_label = 0;
        }
        ProcInst::jump(this, jump_label);
    }
    else{
        call_original!(this, _method_info);
    }
}

//This function creates the animation with the resulting function appearing on-screen alongside their rarity.
#[unity::hook("App", "MapSequenceEngageSummon", "CreateTelop")]
pub fn mapsequenceengagesummon_createtelop(this: &mut MapSequenceEngageSummon, _method_info: OptionalMethod) {
    let mapmind_instance = get_instance::<MapMind>();
    if mapmind_instance.mind == 0x39 {
        return;
    }
    else{
        call_original!(this, _method_info);
    }
}

//This function handles spawning the summoned unit, with all the proper flags.
#[unity::hook("App", "Unit", "CreateForSummon")]
pub fn unit_createforsummon(this: &mut Unit, original: &mut Unit, rank: i32, person: &mut PersonData, _method_info: OptionalMethod) {
    call_original!(this, original, rank, person, _method_info);
    let cur_mind = get_instance::<MapMind>().mind;
    if cur_mind == 0x39 {
        //The status value in question denotes the spawned unit as a summon.
        //We turn this off to keep the summon from de-spawning.
        if (this.status.value & 0x200000000000) != 0 {
            this.status.value = this.status.value ^ 0x200000000000;
            this.update();
            //This code separates the unit from the emblem.
            original.clear_parent();
            original.update();
        }
    }
}

//This function determines which unit to spawn for the summoning.
#[unity::hook("App", "UnitUtil", "CalcSummon")]
pub fn unitutil_calcsummon(person: &mut &mut PersonData, rank: &mut i32, skill: &SkillData, color: i32, dbgrank: i32, _method_info: OptionalMethod) -> bool {
    let mapmind_instance = get_instance::<MapMind>();
    if mapmind_instance.mind == 0x39 {
        *rank = 3;
        let maptarget_instance = get_instance::<MapTarget>();

        let personlist = PersonData::get_list_mut().expect("Couldn't reach PersonData List");
        let personcheck = personlist
        .iter_mut()
        .find(|curr_char|curr_char.pid.to_string() == ("PID_SUMMON_".to_owned() + &maptarget_instance.unit.unwrap().god_unit.unwrap().data.asset_id.to_string()));
        if personcheck.is_some() {
            *person = personcheck.unwrap();
            return true;
        }
        else {
            return false;
        }
    }
    else {
        call_original!(person, rank, skill, color, dbgrank, _method_info)
    }
}

  // Create our new menu command.
#[unity::hook("App", "MapUnitCommandMenu", "CreateBind")]
pub fn mapunitcommandmenu_createbind(sup: &mut ProcInst, _method_info: OptionalMethod) {
    let maptarget_instance = get_instance::<MapTarget>();

    // Create a new class using EngageSummonMenuItem as reference so that we do not wreck the original command for ours.
    let disengage = DISENGAGE_CLASS.get_or_init(|| {
        // EngageSummonMenuItem is a nested class inside of MapUnitCommandMenu, so we need to dig for it.
        let menu_class  = *MapUnitCommandMenu::class()
            .get_nested_types()
            .iter()
            .find(|class| class.get_name().contains("EngageSummonMenuItem"))
            .unwrap();
        
        let new_class = menu_class.clone();

        new_class
            .get_virtual_method_mut("GetName")
            .map(|method| method.method_ptr = disengage_get_name as _)
            .unwrap();

        new_class
            .get_virtual_method_mut("GetCommandHelp")
            .map(|method| method.method_ptr = disengage_get_desc as _)
            .unwrap();

        new_class
            .get_virtual_method_mut("get_Mind")
            .map(|method| method.method_ptr = disengage_get_mind as _)
            .unwrap();

        new_class
            .get_virtual_method_mut("get_FlagID")
            .map(|method| method.method_ptr = disengage_get_flagid as _)
            .unwrap();

        new_class
            .get_virtual_method_mut("GetMapAttribute")
            .map(|method| method.method_ptr = disengage_get_map_attribute as _)
            .unwrap();

        new_class
            .get_virtual_method_mut("get_IsForecast")
            .map(|method| method.method_ptr = disengage_get_is_forecast as _)
            .unwrap();

        new_class
    });

    call_original!(sup, _method_info);

    // Instantiate our custom class as if it was EngageSummonMenuItem
    let instance = Il2CppObject::<EngageSummonMenuItem>::from_class(disengage).unwrap();

    let menu_item_list = &mut sup.child.as_mut().unwrap().cast_mut::<BasicMenu<EngageSummonMenuItem>>().full_menu_item_list;
    menu_item_list.insert((menu_item_list.len() - 1) as i32, instance);
    

}

pub extern "C" fn disengage_get_name(_this: &(), _method_info: OptionalMethod) -> &'static Il2CppString {
    
    if get_instance::<MapTarget>().unit.unwrap().get_god_unit().is_none() {
        return "Reunite".into();
    }
    
    "Separate".into()
}

pub extern "C" fn disengage_get_desc(_this: &(), _method_info: OptionalMethod) -> &'static Il2CppString {
    if get_instance::<MapTarget>().unit.unwrap().get_god_unit().is_none() {
        return "Join up with an Emblem.".into();
    }
    
    "Separate from your Emblem.".into()
}

pub extern "C" fn disengage_get_mind(_this: &(), _method_info: OptionalMethod) -> i32 {
    0x39
}

pub extern "C" fn disengage_get_flagid(_this: &(), _method_info: OptionalMethod) -> &'static Il2CppString {
    "Disengage".into()
}

pub extern "C" fn disengage_get_is_forecast(_this: &(), _method_info: OptionalMethod) -> bool {
    false
}


pub extern "C" fn disengage_get_map_attribute(_this: &(), _method_info: OptionalMethod) -> i32 {
    let maptarget_instance = get_instance::<MapTarget>();
    if maptarget_instance.unit.unwrap().get_god_unit().is_some() {
        return 1;
    }
    else{
        if let Some(dataset) = maptarget_instance.m_dataset.as_mut() {
            dataset.clear();
        }
        maptarget_enumerate(maptarget_instance, 0, _method_info);
        if (maptarget_instance.m_dataset.as_ref().unwrap().fields.m_list.size > 0) && maptarget_instance.unit.unwrap().get_job().name.to_string() != "MJID_Emblem"{
            return 1;
        }
        else {
            return 4;
        }
    }
}

#[skyline::main(name = "DisEngage")]
pub fn main() {
    // Install a panic handler for your plugin, allowing you to customize what to do if there's an issue in your code.
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();

        // Some magic thing to turn what was provided to the panic into a string. Don't mind it too much.
        // The message will be stored in the msg variable for you to use.
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => {
                match info.payload().downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<Any>",
                }
            },
        };

        // This creates a new String with a message of your choice, writing the location of the panic and its message inside of it.
        // Note the \0 at the end. This is needed because show_error is a C function and expects a C string.
        // This is actually just a result of bad old code and shouldn't be necessary most of the time.
        let err_msg = format!(
            "DisEngage has panicked at '{}' with the following message:\n{}\0",
            location,
            msg
        );

        // We call the native Error dialog of the Nintendo Switch with this convenient method.
        // The error code is set to 69 because we do need a value, while the first message displays in the popup and the second shows up when pressing Details.
        skyline::error::show_error(
            69,
            "DisEngage has panicked! Please open the details and send a screenshot to the developer, then close the game.\n\0",
            err_msg.as_str(),
        );
    }));

    skyline::install_hooks!(
        mapunitcommandmenu_createbind,
        maptarget_enumerate,
        mapsequencemind_branch,
        mapsequenceengagesummon_branch,
        unit_createforsummon,
        unitutil_calcsummon,
        mapsummonmenu_createsummonbind,
        mapsequenceengagesummon_createtelop,
        mapsequencetargetselect_decide_normal,
        mapbattleinfoparamsetter_setbattleinfo,
        mapbattleinforoot_setcommandtext,
        mapbattleinforoot_setup,
        mapsequence_complete
    );
}
