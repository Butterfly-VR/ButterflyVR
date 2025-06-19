extends Node3D

var is_grabbing:bool = false
var grabbed_node:Node3D
var owner_id:int
var networker:PlayerNetworker

func _ready() -> void:
	@warning_ignore("unsafe_property_access")
	get_parent().get_parent() .get_parent().remote = get_parent() # this probably shouldnt be here but didnt wanna make a new script for a single line
	@warning_ignore("unsafe_property_access")
	networker = get_parent().get_parent().get_parent().networker
	owner_id = networker.owner_id

func _physics_process(_delta: float) -> void:
	if (NetworkManager as NetNodeManager).has_message() and (NetworkManager as NetNodeManager).get_message_player() == owner_id:
		if (NetworkManager as NetNodeManager).get_message_type() == 1:
			@warning_ignore("unsafe_call_argument")
			on_grab((NetworkManager as NetNodeManager).peek_message()[0])
			(NetworkManager as NetNodeManager).pop_message()
		elif (NetworkManager as NetNodeManager).get_message_type() == 2:
			on_release()
			(NetworkManager as NetNodeManager).pop_message()
	@warning_ignore("unsafe_property_access")
	if networker.get_parent().cam_x_rotation != null:
		@warning_ignore("unsafe_property_access", "unsafe_method_access")
		get_parent().rotate_x(networker.get_parent().cam_x_rotation - get_parent().rotation.x)
	if is_grabbing and grabbed_node != null:
		grabbed_node.global_position = global_position

func on_release() -> void:
	is_grabbing = false
	if grabbed_node is RigidBody3D:
		(grabbed_node as RigidBody3D).freeze = false
	grabbed_node = null

func on_grab(target_path:String) -> void:
	is_grabbing = true
	var target:Node3D = get_node(target_path)
	global_position = target.global_position
	grabbed_node = target
	if grabbed_node is RigidBody3D:
		(grabbed_node as RigidBody3D).freeze = true
