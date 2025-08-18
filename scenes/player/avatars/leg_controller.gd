extends Node3D

const DISTANCE_THRESHOLD_MULTIPLIER:float = 0.5
const TIME_THRESHOLD_MULTIPLIER:float = 1.0
const MOVE_THRESHOLD:float = 0.05
const TARGET_VELOCITY_OFFSET_MULTIPLIER:float = 10.0
const RISE_TRAVEL_TIME:float = 1
const RISE_TARGET_HEIGHT:float = -0.1
const TRAVEL_SPEED:float = 60
const FALL_TRAVEL_TIME:float = 0.06

@export var leg1:GodotIKEffector
var leg1_cycle_state:cycle_states = cycle_states.planted
var leg1_time_since_move:float = 0.0
@export var leg1_target:Node3D
var leg1_velocity_position_offset:Vector3
var leg1_target_base_position:Vector3

@export var leg2:GodotIKEffector
var leg2_cycle_state:cycle_states = cycle_states.planted
var leg2_time_since_move:float = 0.0
@export var leg2_target:Node3D
var leg2_velocity_position_offset:Vector3
var leg2_target_base_position:Vector3

var rising_height_progress:float = 0.0

@onready var target:Node3D = get_parent().get_parent()
var last_target_position:Vector3

enum cycle_states{
	planted,
	rising,
	travelling,
	falling
}

func _physics_process(delta: float) -> void:
	if last_target_position == null:
		last_target_position = target.global_position
		return
	var travel_speed:float = TRAVEL_SPEED * (target.global_position - last_target_position).length()
	
	leg1_target.global_position.y = leg1.global_position.y
	leg2_target.global_position.y = leg2.global_position.y
	
	# todo: get this working
	#if leg1_target_base_position == null:
	#	leg1_target_base_position = leg1_target.position
	#var offset = ((target.global_position - last_target_position) * TARGET_VELOCITY_OFFSET_MULTIPLIER)
	#offset.y = 0
	#leg1_target.position = leg1_target_base_position
	#leg1_target.global_translate(offset)
	
	if leg1_cycle_state != cycle_states.planted:
		leg1.global_rotation.y = target.global_rotation.y
	if leg2_cycle_state != cycle_states.planted:
		leg2.global_rotation.y = target.global_rotation.y
	# if planted calculate new leg positions in global position and update
	if leg1_cycle_state == cycle_states.planted:
		leg1.top_level = true
		var raycast:RayCast3D = leg1.get_child(0)
		raycast.force_raycast_update()
		if !raycast.is_colliding() and leg2_cycle_state == cycle_states.planted:
			leg1.top_level = false
			leg1_cycle_state = cycle_states.rising
			leg1_time_since_move = 0.0
			
		leg1_time_since_move = clampf(leg1_time_since_move + delta, 0, 2.0)
		if leg2_cycle_state == cycle_states.planted:
			var difference_leg1:Vector3 = abs(leg1.global_position - leg1_target.global_position)
			var move_desire_leg1:float = (difference_leg1.length() * DISTANCE_THRESHOLD_MULTIPLIER) * (leg1_time_since_move * TIME_THRESHOLD_MULTIPLIER)
			if move_desire_leg1 > MOVE_THRESHOLD:
				leg1.top_level = false
				leg1_cycle_state = cycle_states.rising
				leg1_time_since_move = 0.0
	if leg2_cycle_state == cycle_states.planted:
		leg2.top_level = true
		var raycast:RayCast3D = leg2.get_child(0)
		raycast.force_raycast_update()
		if !raycast.is_colliding():
			leg2.top_level = false
			leg2_cycle_state = cycle_states.rising
			leg2_time_since_move = 0.0
			
		leg2_time_since_move = clampf(leg2_time_since_move + delta, 0, 2.0)
		if leg1_cycle_state == cycle_states.planted:
			var difference_leg2:Vector3 = abs(leg2.global_position - leg2_target.global_position)
			var move_desire_leg2:float = (difference_leg2.length() * DISTANCE_THRESHOLD_MULTIPLIER) * (leg2_time_since_move * TIME_THRESHOLD_MULTIPLIER)
			if move_desire_leg2 > MOVE_THRESHOLD:
				leg2.top_level = false
				leg2_cycle_state = cycle_states.rising
				leg2_time_since_move = 0.0
	
	# in rising stage move to the target height and forward to at most halfway towards target position
	if leg1_cycle_state == cycle_states.rising:
		var height_multiplier:float = RISE_TARGET_HEIGHT / RISE_TRAVEL_TIME
		
		leg1.global_position.y = move_toward(leg1.global_position.y, leg1.global_position.y + RISE_TARGET_HEIGHT + rising_height_progress, height_multiplier * delta)
		rising_height_progress += maxf(height_multiplier * delta, RISE_TARGET_HEIGHT)
		
		if is_equal_approx(rising_height_progress, RISE_TARGET_HEIGHT):
			leg1_cycle_state = cycle_states.travelling
		
		var multiplier:float = travel_speed
		var max_travel:Vector2 = (Vector2(leg1_target.global_position.x, leg1_target.global_position.z) - Vector2(leg1.global_position.x, leg1.global_position.z)).normalized()
		var travel_amount:Vector2 = multiplier * delta * max_travel
		
		if travel_amount.length_squared() > ((Vector2(leg1_target.global_position.x, leg1_target.global_position.z) - Vector2(leg1.global_position.x, leg1.global_position.z)) / 2).length_squared() or travel_amount.length_squared() == 0:
			leg1.global_position.x = leg1_target.global_position.x
			leg1.global_position.z = leg1_target.global_position.z
			leg1_cycle_state = cycle_states.falling
		else:
			leg1.global_position += Vector3(travel_amount.x, 0, travel_amount.y)
	elif leg2_cycle_state == cycle_states.rising:
		var height_multiplier:float = RISE_TARGET_HEIGHT / RISE_TRAVEL_TIME
		
		leg2.global_position.y = move_toward(leg2.global_position.y, leg2.global_position.y + RISE_TARGET_HEIGHT + rising_height_progress, height_multiplier * delta)
		rising_height_progress += maxf(height_multiplier * delta, RISE_TARGET_HEIGHT)
		
		if is_equal_approx(rising_height_progress, RISE_TARGET_HEIGHT):
			leg2_cycle_state = cycle_states.travelling
		
		var multiplier:float = travel_speed
		var max_travel:Vector2 = (Vector2(leg2_target.global_position.x, leg2_target.global_position.z) - Vector2(leg2.global_position.x, leg2.global_position.z)).normalized()
		var travel_amount:Vector2 = multiplier * delta * max_travel
		
		if travel_amount.length_squared() > ((Vector2(leg2_target.global_position.x, leg2_target.global_position.z) - Vector2(leg2.global_position.x, leg2.global_position.z)) / 2).length_squared() or travel_amount.length_squared() == 0:
			leg2.global_position.x = leg2_target.global_position.x
			leg2.global_position.z = leg2_target.global_position.z
			leg2_cycle_state = cycle_states.falling
		else:
			leg2.global_position += Vector3(travel_amount.x, 0, travel_amount.y)
	
	# in travelling move directly to target ignoring height
	if leg1_cycle_state == cycle_states.travelling:
		var multiplier:float = travel_speed
		var max_travel:Vector2 = (Vector2(leg1_target.global_position.x, leg1_target.global_position.z) - Vector2(leg1.global_position.x, leg1.global_position.z)).normalized()
		var travel_amount:Vector2 = multiplier * delta * max_travel
		
		if travel_amount.length_squared() > ((Vector2(leg1_target.global_position.x, leg1_target.global_position.z) - Vector2(leg1.global_position.x, leg1.global_position.z)) / 2).length_squared() or travel_amount.length_squared() == 0:
			leg1.global_position.x = leg1_target.global_position.x
			leg1.global_position.z = leg1_target.global_position.z
			leg1_cycle_state = cycle_states.falling
		else:
			leg1.global_position += Vector3(travel_amount.x, 0, travel_amount.y)
	elif leg2_cycle_state == cycle_states.travelling:
		var multiplier:float = travel_speed
		var max_travel:Vector2 = (Vector2(leg2_target.global_position.x, leg2_target.global_position.z) - Vector2(leg2.global_position.x, leg2.global_position.z)).normalized()
		var travel_amount:Vector2 = multiplier * delta * max_travel
		
		if travel_amount.length_squared() > ((Vector2(leg2_target.global_position.x, leg2_target.global_position.z) - Vector2(leg2.global_position.x, leg2.global_position.z)) / 2).length_squared() or travel_amount.length_squared() == 0:
			leg2.global_position.x = leg2_target.global_position.x
			leg2.global_position.z = leg2_target.global_position.z
			leg2_cycle_state = cycle_states.falling
		else:
			leg2.global_position += Vector3(travel_amount.x, 0, travel_amount.y)

	# in falling move down to target, once at target return to planted
	if leg1_cycle_state == cycle_states.falling:
		var height_multiplier:float = RISE_TARGET_HEIGHT / FALL_TRAVEL_TIME
		
		leg1.global_position.y += height_multiplier * delta
		
		var raycast:RayCast3D = leg1.get_child(0) as RayCast3D
		raycast.force_raycast_update()
		
		if raycast.is_colliding():
			rising_height_progress = 0.0
			leg1_cycle_state = cycle_states.planted
		
		var multiplier:float = 1.0 / FALL_TRAVEL_TIME
		var max_travel:Vector2 = (Vector2(leg1_target.global_position.x, leg1_target.global_position.z) - Vector2(leg1.global_position.x, leg1.global_position.z)).normalized()
		var travel_amount:Vector2 = multiplier * delta * max_travel
		
		if travel_amount.length_squared() > (Vector2(leg1_target.global_position.x, leg1_target.global_position.z) - Vector2(leg1.global_position.x, leg1.global_position.z)).length_squared() or travel_amount.length_squared() == 0:
			leg1.global_position.x = leg1_target.global_position.x
			leg1.global_position.z = leg1_target.global_position.z
		else:
			leg1.global_position += Vector3(travel_amount.x, 0, travel_amount.y)
	elif leg2_cycle_state == cycle_states.falling:
		var height_multiplier:float = RISE_TARGET_HEIGHT / FALL_TRAVEL_TIME
		leg2.global_position.y += height_multiplier * delta
		
		var raycast:RayCast3D = leg2.get_child(0) as RayCast3D
		raycast.force_raycast_update()
		
		if raycast.is_colliding():
			rising_height_progress = 0.0
			leg2_cycle_state = cycle_states.planted
		
		var multiplier:float = 1.0 / FALL_TRAVEL_TIME
		var max_travel:Vector2 = (Vector2(leg2_target.global_position.x, leg2_target.global_position.z) - Vector2(leg2.global_position.x, leg2.global_position.z)).normalized()
		var travel_amount:Vector2 = multiplier * delta * max_travel
		
		if travel_amount.length_squared() > (Vector2(leg2_target.global_position.x, leg2_target.global_position.z) - Vector2(leg2.global_position.x, leg2.global_position.z)).length_squared() or travel_amount.length_squared() == 0:
			leg2.global_position.x = leg2_target.global_position.x
			leg2.global_position.z = leg2_target.global_position.z
		else:
			leg2.global_position += Vector3(travel_amount.x, 0, travel_amount.y)
	
	last_target_position = target.global_position
