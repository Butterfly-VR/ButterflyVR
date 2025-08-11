extends RayCast3D

@onready var grab_target:Node3D = get_child(0)

var grab_handler:PlayerGrabHandler = GlobalWorldAccess.current_world.player_grab_handler
var is_grabbing:bool = false
var wants_to_grab:bool = false
var grabbed_node:Node3D

func _ready() -> void:
	GlobalWorldAccess.current_world.player_grab_handler.player_grabbed.connect(on_confirmed_grab)

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("player_grab"):
		wants_to_grab = true
	if event.is_action_released("player_grab"):
		is_grabbing = false
		wants_to_grab = false

func _physics_process(_delta: float) -> void:
	if wants_to_grab:
		wants_to_grab = false
		force_raycast_update()
		if is_colliding():
			grab_handler.send_message((NetworkManager as NetNodeManager).get_id(), (get_collider() as Node).get_path())
		return
	if is_grabbing:
		grabbed_node.global_position = grab_target.global_position
	else:
		if grabbed_node != null:
			if grabbed_node is RigidBody3D:
				(grabbed_node as RigidBody3D).freeze = false
			grabbed_node = null
			grab_target.position = Vector3.ZERO
			grab_handler.send_message((NetworkManager as NetNodeManager).get_id(), "")
			# todo: enabling this seems to cause on_confirmed_grab / on_grab to trigger twice, with the second call failing to get_node despite the path definetly being valid? this is very weird
			#if get_collider().has_method("on_release"):
			#	get_collider().on_release()

func on_confirmed_grab(player:int, target:Node) -> void:
	if !(player == (NetworkManager as NetNodeManager).get_id()):
		return
	if target is Node3D:
		is_grabbing = true
		grab_target.global_position = target.global_position
		grabbed_node = target
		if grabbed_node is RigidBody3D:
			(grabbed_node as RigidBody3D).freeze = true
