#![feature(ptr_sub_ptr)]

mod enume;
use std::num::*;
use enume::ReUniteTargetEnumerator;
use skyline::println;

use std::sync::OnceLock;

use mapunitcommand::MapUnitCommandMenu;
use unity::prelude::*;

use engage::{
    force::{Force, ForceType}, gamedata::{skill::SkillData, unit::{GodUnit, Unit}, Gamedata, GodData, PersonData}, gamesound::GameSound, mapmind::MapMind, menu::*, proc::{Bindable, ProcInst, ProcInstFields}, sequence::{
        mapsequence::human::MapSequenceHuman,
        mapsequencetargetselect::{MapSequenceTargetSelect, MapTarget}
    }, unitpool::UnitPool, util::{get_instance, get_singleton_proc_instance}
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

#[skyline::from_offset(0x01a0c990)]
fn unit_item_add_from_iid(this: &Unit, iid: &'static Il2CppString, _method_info: OptionalMethod);

#[skyline::from_offset(0x01a41450)]
fn unit_item_put_off_from_index(this: &Unit, index: i32, closeup: bool, _method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "CanEngageStart")]
fn unit_can_engage_start(this: &Unit, _method_info: OptionalMethod) -> bool;

#[skyline::from_offset(0x01e8d5b0)]
fn battlemath_get_value(num: i32, _method_info: OptionalMethod) -> i32;

static DISENGAGE_CLASS: OnceLock<&'static mut Il2CppClass> = OnceLock::new();

pub trait UnitItemManip {
    fn unit_item_add_iid(&mut self, iid: &'static Il2CppString);
    fn unit_item_remove_index(&mut self, index: i32, closeup: bool);
}
  
impl UnitItemManip for Unit {
    fn unit_item_add_iid(&mut self, iid: &'static Il2CppString) {
        unsafe{unit_item_add_from_iid(self, iid, None as OptionalMethod)};
    }
    fn unit_item_remove_index(&mut self, index: i32, closeup: bool) {
        unsafe{unit_item_put_off_from_index(self, index, closeup, None as OptionalMethod)};
    }
}

pub trait UnitEngageManip {
    fn unit_engage_check(&self) -> bool;
    fn unit_can_engage(&self) -> bool;
}

impl UnitEngageManip for Unit {
    fn unit_engage_check(&self) -> bool {
        if self.status.value & 0x2000000 == 0 {
            return false;
        }
        true
    }
    fn unit_can_engage(&self) -> bool {
        unsafe{unit_can_engage_start(self, None)}
    }
}

///This function is used for creating the unit.
///We edit it here in order to properly fill out it's inventory.
#[unity::hook("App", "Unit", "CreateForSummonImpl1")]
pub fn unit_createforsummonimpl1_disengage(this: &mut Unit, person: &PersonData, original: &Unit, rank: i32, method_info: OptionalMethod) {
    call_original!(this, person, original, rank, method_info);
    let map_target = get_instance::<MapTarget>();
    if map_target.m_mind == 0x39 {
        this.unit_item_remove_index(0, false);
        if person.get_items().is_some() {
            let orig_level = original.level + original.internal_level as u8;
            let item_count = (person.get_items().unwrap().len() - 1) as u8;
            let item_give =  ((orig_level / 10).clamp(0, item_count)) as usize;
            
            let mut index = 0;
            loop {
                this.unit_item_add_iid(person.get_items().unwrap().get(index).unwrap());
                if index == item_give {
                    break;
                }
                index = index + 1;
            }
        }
    }
}

/// This function is what sets the text that appears in between the two windows
/// when targetting another unit.
#[unity::hook("App", "MapBattleInfoRoot", "SetCommandText")]
pub fn mapbattleinforoot_setcommandtext_disengage(this: &mut MapBattleInfoRoot, mind_type: i32, method_info: OptionalMethod) {
    if mind_type != 0x39 {
        call_original!(this, mind_type, method_info);
    } else {
        let map_target = get_instance::<MapTarget>();

        if map_target.unit.unwrap().get_god_unit().is_some() {
            InfoUtil::try_set_text(&this.command_text, "Separate");
        } else {
            InfoUtil::try_set_text(&this.command_text, "Re-Unite");
        }
    }
}

/// Makes the game hide the damage forecast arrows.
/// This function is primarily for setting the
/// command name in between the two windows, and deciding whether to hide the damage arrows.
/// Thankfully, the default behavior is almost exactly what we want, we just need to adjust it
/// to return false, since that's what hides the damage arrows.
#[unity::hook("App", "MapBattleInfoRoot", "Setup")]
pub fn mapbattleinforoot_setup_disengage(this: &(), mindtype: i32, skill: &SkillData, info: &(), scene_list: &(), method_info: OptionalMethod) -> bool {
    let result = call_original!(this, mindtype, skill, info, scene_list, method_info);

    if mindtype == 0x39 {
        false
    } else {
        result
    }
}


/// This function is responsible for the windows that pop up when you highlight a target.
/// The default behavior without this hook makes the battle forecast appear.  So weapons, hp, etc.
#[unity::hook("App", "MapBattleInfoParamSetter", "SetBattleInfo")]
pub fn mapbattleinfoparamsetter_setbattleinfo_disengage(this: &mut MapBattleInfoParamSetter, side_type: i32, show_window: bool, battle_info: &(), scene_list: &(), method_info: OptionalMethod) {
    call_original!(this, side_type, show_window, battle_info, scene_list, method_info);

    let map_target = get_instance::<MapTarget>();

    if map_target.m_mind == 0x39 && map_target.unit.unwrap().get_god_unit().is_none() {
        this.set_battle_info_for_no_param(false, false);
    }
}


/// This is a generic function that essentially checks the Mind value, and then calls
/// a more specialized Enumerate function based on the result.
/// Enumerate functions are used for checking if there is a valid target in range,
/// and making a list of them.
#[unity::hook("App", "MapTarget", "Enumerate")]
pub fn maptarget_enumerate_disengage(this: &mut MapTarget, mask: i32, method_info: OptionalMethod) {
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
            unsafe { maptarget_enumerateselfonly(this, 0, None) };
        } else {
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
    } else {
        call_original!(this, mask, method_info);
    }
}


/// This is the function that usually runs when you press A while highlighting a target and the
/// forecast windows are up.
#[unity::hook("App", "MapSequenceTargetSelect", "DecideNormal")]
pub fn mapsequencetargetselect_decide_normal_disengage(this: &mut MapSequenceTargetSelect, method_info: OptionalMethod) {
    let maptarget_instance = get_instance::<MapTarget>();


    let cur_mind = maptarget_instance.m_mind;

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
                                curr_god.get_gid().unwrap().to_string() == ("GID_".to_owned() + &unsafe { persondata_getsummongod(person, method_info) }.unwrap().to_string())
                            });
                        
                        if let Some(god) = god_data {
                            println!("God: {}", god.ascii_name.unwrap());

                            if let Some(god_unit) = unsafe { godpool_tryget(god.get_gid().unwrap(), method_info) } {
                                println!("God: {}", god_unit.data.ascii_name.unwrap());

                                maptarget_instance.unit.unwrap().set_god_unit(god_unit);
                                maptarget_instance.unit.unwrap().update();

                                unsafe { unitutil_summondeleteimpl(target.m_unit, method_info); }
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
        call_original!(this, method_info)
    }
}

///This is the function that builds the summon menu allowing you to pick your desired orb color.
///This hook is here to skip that menu.
#[unity::hook("App", "MapSummonMenu", "CreateSummonBind")]
pub fn mapsummonmenu_createsummonbind_disengage(sup: &mut ProcInst, method_info: OptionalMethod) {
    let map_target = get_instance::<MapTarget>();
    
    if map_target.m_mind == 0x39 {
        let map_mind = get_instance::<MapMind>();
        map_mind.item_index = 1;

        let unit = map_target.unit.expect("MapTarget does not have a Unit");
        map_mind.x = unit.x as i8;
        map_mind.z = unit.z as i8;

        let map_sequence_human = get_singleton_proc_instance::<MapSequenceHuman>().unwrap();

        // Jump to MapSequenceHuman::TargetSelect
        ProcInst::jump(map_sequence_human,0x20);
    }
    else {
        call_original!(sup, method_info);
    }
}

/// This function reads your mind value, and ProcInst::Jump to the desired proc location.
#[unity::hook("App", "MapSequenceMind", "Branch")]
pub fn mapsequencemind_branch_disengage(this: &mut MapSequenceMind, method_info: OptionalMethod) {
    call_original!(this, method_info);

    let map_mind = get_instance::<MapMind>();

    // Custom value for our command
    if map_mind.mind == 0x39 {
        // Jump to MapSequenceMind::EngageSummon
        ProcInst::jump(this, 0x16);
    }
}

/// This function determines if you have combat anims on, and then ProcInst::Jump
/// to the appropriate part of MapSequenceEngageSummon's procs based on that.
/// This hook is for making it always use the non-anims setup.
#[unity::hook("App", "MapSequenceEngageSummon", "Branch")]
pub fn mapsequenceengagesummon_branch_disengage(this: &mut MapSequenceEngageSummon, method_info: OptionalMethod) {
    let map_mind = get_instance::<MapMind>();

    if map_mind.mind == 0x39 {
        let jump_label = if unsafe { fade_isfadeout(method_info) } {
            2 // MapSequenceEngageSummon::After
        } else {
            0 // MapSequenceEngageSummon::Simple
        };

        ProcInst::jump(this, jump_label);
    } else {
        call_original!(this, method_info);
    }
}

/// This function creates the animation with the resulting function appearing on-screen alongside their rarity.
/// This hook is simply to skip that little animation.
#[unity::hook("App", "MapSequenceEngageSummon", "CreateTelop")]
pub fn mapsequenceengagesummon_createtelop_disengage(this: &mut MapSequenceEngageSummon, method_info: OptionalMethod) {
    let map_mind = get_instance::<MapMind>();
    
    if map_mind.mind != 0x39 {
        call_original!(this, method_info);
    }
}

/// This function handles spawning the summoned unit, with all the proper flags.
#[unity::hook("App", "Unit", "CreateForSummon")]
pub fn unit_createforsummon_disengage(this: &mut Unit, original: &mut Unit, rank: i32, person: &mut PersonData, method_info: OptionalMethod) {
    call_original!(this, original, rank, person, method_info);

    let map_mind = get_instance::<MapMind>();

    if map_mind.mind == 0x39 {
        // The status value in question denotes the spawned unit as a summon.
        // We turn this off to keep the summon from de-spawning.
        if (this.status.value & 0x200000000000) != 0 {
            this.status.value = this.status.value ^ 0x200000000000;
            //Turns on the DisposGuest status so that the unit is removed
            //from the player's units after the map ends.
            this.status.value = this.status.value ^ 0x400000;
            this.update();

            //This code separates the unit from the emblem.
            original.clear_parent();
            original.update();
        }
    }
}

/// This function determines which unit to spawn for the summoning.
#[unity::hook("App", "UnitUtil", "CalcSummon")]
pub fn unitutil_calcsummon_disengage(person: &mut &mut PersonData, rank: &mut i32, skill: &SkillData, color: i32, dbgrank: i32, method_info: OptionalMethod) -> bool {
    let map_mind = get_instance::<MapMind>();

    if map_mind.mind == 0x39 {
        *rank = 3;

        let map_target = get_instance::<MapTarget>();

        let personlist = PersonData::get_list_mut().expect("Couldn't reach PersonData List");
        let mut god_name = map_target.unit.unwrap().god_unit.unwrap().data.asset_id.to_string();

        //Add the possibility of getting Robin instead of Chrom
        if god_name == "クロム" {
            let rnmnm = unsafe{battlemath_get_value(2, method_info)};
            println!("{}", rnmnm);
            if rnmnm != 1 {
                god_name = "ルフレ".to_string();
            };
        }
        ///If you have Ephraim equipped, this'll give you Eirika instead.
        ///Feel free to remove this if you've added an Ephraim Disengage PID.
        else if god_name == "エフラム" {
            god_name = "エイリーク".to_string();
        }
            
        let person_data = personlist
            .iter_mut()
            .find(|curr_char|curr_char.pid.to_string() == ("PID_DISENGAGE_".to_owned() + &god_name)); // Ray: I'd usually smithe someone from writing this many unwraps without handling errors, but I assume if this runs into an error it should crash anyways.
        if let Some(found_person) = person_data {
            *person = found_person;
            true
        } else {
            false
        }
    } else {
        call_original!(person, rank, skill, color, dbgrank, method_info)
    }
}

  // Create our new menu command.
#[unity::hook("App", "MapUnitCommandMenu", "CreateBind")]
pub fn mapunitcommandmenu_createbind_disengage(sup: &mut ProcInst, _method_info: OptionalMethod) {
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
        "Reunite".into()
    } else {
        "Separate".into()
    }
}

pub extern "C" fn disengage_get_desc(_this: &(), _method_info: OptionalMethod) -> &'static Il2CppString {
    if get_instance::<MapTarget>().unit.unwrap().get_god_unit().is_none() {
        "Join up with an Emblem.".into()
    } else {
        "Separate from your Emblem.".into()
    }
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

///Determines if the option appears in the menu.
///returning 1 means it appears, returning 4 means it does not.
pub extern "C" fn disengage_get_map_attribute(_this: &(), _method_info: OptionalMethod) -> i32 {
    let map_target = get_instance::<MapTarget>();

    if map_target.unit.is_none() {
        return 4;
    }
    if map_target.unit.unwrap().get_god_unit().is_some() && map_target.unit.unwrap().unit_engage_check() == false && map_target.unit.unwrap().unit_can_engage() {
        1
    } else {
        if let Some(dataset) = map_target.m_dataset.as_mut() {
            dataset.clear();
        }
        map_target.enumerate_reunite();

        if (map_target.m_dataset.as_ref().unwrap().fields.m_list.size > 0) && map_target.unit.unwrap().get_job().name.to_string() != "MJID_Emblem" && map_target.unit.unwrap().unit_engage_check() == false && map_target.unit.unwrap().status.value & 0x40000 == 0 {
            1
        } else {
            4
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
        mapunitcommandmenu_createbind_disengage,
        maptarget_enumerate_disengage,
        mapsequencemind_branch_disengage,
        mapsequenceengagesummon_branch_disengage,
        unit_createforsummon_disengage,
        unitutil_calcsummon_disengage,
        mapsummonmenu_createsummonbind_disengage,
        mapsequenceengagesummon_createtelop_disengage,
        mapsequencetargetselect_decide_normal_disengage,
        mapbattleinfoparamsetter_setbattleinfo_disengage,
        mapbattleinforoot_setcommandtext_disengage,
        mapbattleinforoot_setup_disengage,
        unit_createforsummonimpl1_disengage
    );
}
