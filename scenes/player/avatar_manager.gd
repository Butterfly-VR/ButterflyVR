extends Node

signal avatar_loaded

@export var equiped_avatar:Node3D
@onready var default_transform:Transform3D = equiped_avatar.transform
var networker:PlayerNetworker
var current_avatar:int
var new_avatar:PackedScene

func _ready() -> void:
	@warning_ignore("unsafe_property_access")
	networker = get_parent().get_parent().networker
	if equiped_avatar:
		setup_avatar(equiped_avatar)

func _physics_process(_delta: float) -> void:
	if (NetworkManager as NetNodeManager).has_message() and (NetworkManager as NetNodeManager).get_message_type() == 5 and (NetworkManager as NetNodeManager).get_message_player() == networker.owner_id:
		var values:Array = (NetworkManager as NetNodeManager).peek_message()
		@warning_ignore("unsafe_call_argument")
		change_avatar(values[0])
		(NetworkManager as NetNodeManager).pop_message()

func change_avatar(avatar:int) -> void:
	if equiped_avatar != null:
		equiped_avatar.queue_free()
	if avatar == 0:
		new_avatar = preload("res://scenes/player/avatars/built in avatars/cubefella/cube_fella.tscn")
	else:
		new_avatar = preload("res://scenes/player/avatars/built in avatars/loading_avatar.tscn")
		current_avatar = avatar
		equiped_avatar = new_avatar.instantiate()
		get_parent().add_child(equiped_avatar)
		equiped_avatar.transform = default_transform
		AvatarPackLoader.update_avatar_list()
		var thread:Thread = Thread.new()
		thread.start(load_avatar_on_thread.bind(avatar))
		await avatar_loaded
		thread.wait_to_finish()
	current_avatar = avatar
	equiped_avatar.queue_free()
	await get_tree().physics_frame # dont have both avatars loaded at the same time
	equiped_avatar = setup_avatar(new_avatar.instantiate())
	get_parent().add_child(equiped_avatar)
	equiped_avatar.transform = default_transform

func load_avatar_on_thread(avatar:int) -> void:
	if !AvatarPackLoader.avatars.has(avatar):
		new_avatar = preload("res://scenes/player/avatars/built in avatars/missing_avatar.tscn")
		avatar_loaded.emit()
		return
	new_avatar = load(AvatarPackLoader.avatars[avatar].scene_path)
	# abort if source avatar is unsafe unless unsafe loading is enabled
	if (!check_safe(new_avatar.get_state())):
		if !OS.get_cmdline_args().has("--unsafe-load"):
			push_warning("***DANGER*** tried to load unsafe avatar! aborting. to override this run with \"--unsafe-load\"")
			new_avatar = preload("res://scenes/player/avatars/built in avatars/missing_avatar.tscn")
		else:
			push_warning("safety check disabled: loading unsafe avatar, this is very dangerous!")
	avatar_loaded.emit.call_deferred()

# checks if the scene is capable of code execution, probably not foolproof
func check_safe(root:SceneState) -> bool:
	# check for scripts
	for idx:int in range(root.get_node_count()):
		for property_idx:int in range(root.get_node_property_count(idx)):
			if root.get_node_property_name(idx, property_idx) == "script":
				if root.get_node_property_value(idx, property_idx) != null:
					return false
	return true

func get_node_and_children_recursive(root:Node) -> Array[Node]:
	var nodes:Array[Node]
	for node:Node in root.get_children():
		nodes.append_array(get_node_and_children_recursive(node))
		nodes.append(node)
	return nodes

# goes through the avatar scene looking for stubs and replaces them with the corrosponding scenes
func setup_avatar(root:Node) -> Node:
	var nodes:Array[Node] = get_node_and_children_recursive(root)
	# check for markers and setup if they exist
	for node:Node in nodes:
		if node.has_meta("IKMarker"):
			var ik:GodotIK = preload("res://scenes/player/avatars/godot_ik.tscn").instantiate()
			var bones:Dictionary = node.get_meta("IKMarker")
			# todo: check marker is valid
			@warning_ignore("unsafe_method_access")
			ik.setup(bones["head_bone"], bones["left_arm_bone"], bones["right_arm_bone"], bones["left_leg_bone"], bones["right_leg_bone"])
			node.add_child(ik)
	return root
