extends Node
class_name NetworkNodeBuilder

@export var prefabs:Dictionary = {0: "res://scenes/player/player.tscn"} # todo: add types when updating godot
@export var path_offsets:Dictionary = {0: 1}
var instances:Dictionary
@export var world:WorldController

func _ready() -> void:
	for p:int in prefabs.keys():
		var x:String = prefabs[p]
		instances[p] = load(x)

func _physics_process(_delta: float) -> void:
	if (NetworkManager as NetNodeManager).has_message() and (NetworkManager as NetNodeManager).get_message_type() == 0:
		var values:Array = (NetworkManager as NetNodeManager).peek_message()
		@warning_ignore("unsafe_call_argument")
		create_node(values[0], values[1], values[2], values[3])
		(NetworkManager as NetNodeManager).pop_message()
func create_node(type:int, owner_id:int, object_id:int, scene_path:String) -> void:
	if instances.has(type):
		@warning_ignore("unsafe_cast")
		var instance:Node = (instances[type] as PackedScene).instantiate()
		@warning_ignore("unsafe_property_access")
		instance.networker.owner_id = owner_id
		@warning_ignore("unsafe_property_access")
		instance.networker.objectid = object_id
		var scene_path_original:String = scene_path
		var temp:PackedStringArray = scene_path.split("/")
		temp.reverse()
		for x:int in range(path_offsets[type]):
			temp.remove_at(0) # this sucks so much but i dont care
		temp.reverse()
		scene_path = "/".join(temp)
		while !has_node(scene_path):
			if scene_path.length() == 0:
				push_warning("invalid path: ", scene_path_original)
			scene_path = scene_path.erase(scene_path.length() - 1)
			while true:
				if scene_path.length() == 0:
					scene_path = "/"
					break
				var c:String = scene_path[scene_path.length() - 1]
				if c == "/":
					break
				scene_path = scene_path.erase(scene_path.length() - 1)
		get_node(scene_path).add_child(instance)
