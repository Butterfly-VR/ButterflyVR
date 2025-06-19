extends Node

@onready var target:MovementHandler = get_parent().get_parent()
var is_moving_forwards:bool = false
var is_moving_backwards:bool = false
var is_moving_left:bool = false
var is_moving_right:bool = false

func _unhandled_input(event:InputEvent) -> void:
	if event.is_action_pressed("player_move_left"):
		is_moving_left = true
	if event.is_action_released("player_move_left"):
		is_moving_left = false
	if event.is_action_pressed("player_move_right"):
		is_moving_right = true
	if event.is_action_released("player_move_right"):
		is_moving_right = false
	if event.is_action_pressed("player_move_forward"):
		is_moving_forwards = true
	if event.is_action_released("player_move_forward"):
		is_moving_forwards = false
	if event.is_action_pressed("player_move_backwards"):
		is_moving_backwards = true
	if event.is_action_released("player_move_backwards"):
		is_moving_backwards = false
	
	if event.is_action_pressed("player_jump"):
		target.should_jump = true
	if event.is_action_released("player_jump"):
		target.should_jump = false
	
	if event.is_action_pressed("player_crouch") and target.player_state == target.Player_states.NONE:
		target.player_state = target.Player_states.CROUCHED
	if event.is_action_released("player_crouch") and target.player_state == target.Player_states.CROUCHED:
		target.player_state = target.Player_states.NONE
	if event.is_action_pressed("player_sprint") and target.player_state == target.Player_states.NONE:
		target.player_state = target.Player_states.SPRINTING
	if event.is_action_released("player_sprint") and target.player_state == target.Player_states.SPRINTING:
		target.player_state = target.Player_states.NONE

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta: float) -> void:
	target.input_dir = Vector2(int(is_moving_right) - int(is_moving_left), int(is_moving_backwards) - int(is_moving_forwards))
