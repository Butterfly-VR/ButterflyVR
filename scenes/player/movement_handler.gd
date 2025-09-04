extends Node
class_name MovementHandler

enum Player_states{
	NONE,
	SPRINTING,
	CROUCHED
}

const SPEED:float = 50.0
const MAX_SPEED:float = 4.0
const JUMP_VELOCITY:float = 6
const CROUCH_SPEED_MULT:float = 0.6
const SPRINT_SPEED_MULT:float = 2.0
const CROUCH_JUMP_MULT:float = 0.6
const CROUCHED_CAMERA_HEIGHT_MULTIPLIER:float = 0.7

@export var player_access:PlayerAccess
@export var camera_rotation_origin:Node3D
var gravity:float = ProjectSettings.get_setting("physics/3d/default_gravity")
var should_jump:bool
var player_state:Player_states = Player_states.NONE
@onready var camera_height:float = camera_rotation_origin.position.y
@onready var target:Player = player_access.player

func _unhandled_input(event:InputEvent) -> void:
	if event.is_action_pressed("player_jump"):
		should_jump = true
	if event.is_action_released("player_jump"):
		should_jump = false
	
	if event.is_action_pressed("player_crouch") and player_state == Player_states.NONE:
		player_state = Player_states.CROUCHED
	if event.is_action_released("player_crouch") and player_state == Player_states.CROUCHED:
		player_state = Player_states.NONE
	if event.is_action_pressed("player_sprint") and player_state == Player_states.NONE:
		player_state = Player_states.SPRINTING
	if event.is_action_released("player_sprint") and player_state == Player_states.SPRINTING:
		player_state = Player_states.NONE

func _physics_process(delta: float) -> void:
	var modified_speed:float = SPEED
	var modified_max_speed:float = MAX_SPEED
	var modified_jump_velocity:float = JUMP_VELOCITY
	
	match player_state:
		Player_states.SPRINTING:
			modified_max_speed = modified_max_speed * SPRINT_SPEED_MULT
			modified_speed = modified_speed * SPRINT_SPEED_MULT
		Player_states.CROUCHED:
			modified_max_speed = modified_max_speed * CROUCH_SPEED_MULT
			modified_speed = modified_speed * CROUCH_SPEED_MULT
			modified_jump_velocity = modified_jump_velocity * CROUCH_JUMP_MULT
	if player_state == Player_states.CROUCHED:
		camera_rotation_origin.position.y = lerp(camera_rotation_origin.position.y, camera_height * CROUCHED_CAMERA_HEIGHT_MULTIPLIER, 0.05)
	else:
		camera_rotation_origin.position.y = lerp(camera_rotation_origin.position.y, camera_height, 0.05)
	
	if not target.is_on_floor():
		target.velocity.y -= gravity * delta
	
	if should_jump and target.is_on_floor():
		target.velocity.y = modified_jump_velocity
		should_jump = false
	
	var input_dir:Vector2 = Input.get_vector("player_move_left", "player_move_right", "player_move_forward", "player_move_backwards")
	input_dir = input_dir.rotated(-camera_rotation_origin.rotation.y)
	var direction:Vector3 = (target.transform.basis * Vector3(input_dir.x, 0, input_dir.y))
	target.velocity.x = move_toward(target.velocity.x, direction.x * modified_max_speed, modified_speed * delta)
	target.velocity.z = move_toward(target.velocity.z, direction.z * modified_max_speed, modified_speed * delta)
