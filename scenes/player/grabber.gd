extends RayCast3D

@onready var grab_target:Node3D = get_child(0)

var is_grabbing:bool = false
var wants_to_grab:bool = false
var grabbed_node:Node3D

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("player_grab"):
		wants_to_grab = true
	if event.is_action_released("player_grab"):
		is_grabbing = false
		wants_to_grab = false

func _physics_process(_delta: float) -> void:
	if (NetworkManager as NetNodeManager).has_message() and (NetworkManager as NetNodeManager).get_message_player() == (NetworkManager as NetNodeManager).get_id():
		if (NetworkManager as NetNodeManager).get_message_type() == 1:
			@warning_ignore("unsafe_call_argument")
			on_confirmed_grab((NetworkManager as NetNodeManager).peek_message()[0])
			(NetworkManager as NetNodeManager).pop_message()
		elif (NetworkManager as NetNodeManager).get_message_type() == 2:
			(NetworkManager as NetNodeManager).pop_message()
	
	if wants_to_grab:
		wants_to_grab = false
		force_raycast_update()
		if is_colliding():
			@warning_ignore("unsafe_call_argument")
			(NetworkManager as NetNodeManager).network_grab(get_collider())
			# todo: enabling this seems to cause on_confirmed_grab / on_grab to trigger twice, with the second call failing to get_node despite the path definetly being valid? this is very weird
			#if get_collider().has_method("on_grab"):
			#	get_collider().on_grab()
		return
	if is_grabbing:
		grabbed_node.global_position = grab_target.global_position
	else:
		if grabbed_node != null:
			if grabbed_node is RigidBody3D:
				(grabbed_node as RigidBody3D).freeze = false
			grabbed_node = null
			grab_target.position = Vector3.ZERO
			(NetworkManager as NetNodeManager).network_release()
			# todo: enabling this seems to cause on_confirmed_grab / on_grab to trigger twice, with the second call failing to get_node despite the path definetly being valid? this is very weird
			#if get_collider().has_method("on_release"):
			#	get_collider().on_release()

func on_confirmed_grab(target_path:String) -> void:
	is_grabbing = true
	var target:Node3D = get_node(target_path)
	grab_target.global_position = target.global_position
	grabbed_node = target
	if grabbed_node is RigidBody3D:
		(grabbed_node as RigidBody3D).freeze = true
