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
	GlobalWorldAccess.current_world.player_grab_handler.player_grabbed.connect(on_grab)

func _physics_process(_delta: float) -> void:
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
		(grabbed_node as RigidBody3D).sleeping = false
	grabbed_node = null

func on_grab(player:int, target:Node) -> void:
	if !(player == owner_id):
		return
	if target == null:
		on_release()
		return
	if target is Node3D:
		is_grabbing = true
		global_position = target.global_position
		grabbed_node = target
		if grabbed_node is RigidBody3D:
			(grabbed_node as RigidBody3D).freeze = true
