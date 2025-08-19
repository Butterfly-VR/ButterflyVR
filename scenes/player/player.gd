extends CharacterBody3D
class_name MovementHandler

enum Player_states{
	NONE,
	SPRINTING,
	CROUCHED
}

const SPEED:float = 50.0 # how fast the player accelerates
const MAX_SPEED:float = 4.0 # the max speed of the player
const JUMP_VELOCITY:float = 6
const CROUCH_SPEED_MULT:float = 0.6
const SPRINT_SPEED_MULT:float = 2.0
const CROUCH_JUMP_MULT:float = 0.6
var camera_height:float
const CROUCHED_CAMERA_HEIGHT_MULTIPLIER:float = 0.7

@export var networker:PlayerNetworker
@export var collider:CollisionShape3D

var gravity:float = ProjectSettings.get_setting("physics/3d/default_gravity")
var head:Node3D
var player:PlayerAccess
var is_local:bool

var should_jump:bool
var player_state:Player_states = Player_states.NONE
var input_dir:Vector2
var cam_x_rotation:float

func init_local() -> void:
	is_local = true
	player = preload("res://scenes/player/local_player_desktop.tscn").instantiate()
	final_init(player)

func init_remote() -> void:
	is_local = false
	player = preload("res://scenes/player/remote_player.tscn").instantiate()
	final_init(player)

func final_init(player:PlayerAccess) -> void:
	player.player = self
	player.networker = networker
	add_child.call_deferred(player)

func _physics_process(delta:float) -> void:
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
	if head:
		if player_state == Player_states.CROUCHED:
			head.position.y = lerp(head.position.y, camera_height * CROUCHED_CAMERA_HEIGHT_MULTIPLIER, 0.05)
		else:
			head.position.y = lerp(head.position.y, camera_height, 0.05)
	
	if not is_on_floor():
		velocity.y -= gravity * delta
	
	if should_jump and is_on_floor():
		velocity.y = modified_jump_velocity
		should_jump = false
	
	var direction:Vector3 = (transform.basis * Vector3(input_dir.x, 0, input_dir.y)).normalized()
	if direction.x:
		velocity.x = move_toward(velocity.x, direction.x * modified_max_speed, modified_speed * delta * absf(direction.x))
	else:
		velocity.x = move_toward(velocity.x, 0, modified_speed * delta * absf(velocity.normalized().x))
	if direction.z:
		velocity.z = move_toward(velocity.z, direction.z * modified_max_speed, modified_speed * delta * absf(direction.z))
	else:
		velocity.z = move_toward(velocity.z, 0, modified_speed * delta * absf(velocity.normalized().z))

	move_and_slide()
