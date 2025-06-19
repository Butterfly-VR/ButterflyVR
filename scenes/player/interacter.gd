extends RayCast3D

var is_interacting:bool = false
var wants_to_interact_primary:bool = false
var wants_to_interact_secondary:bool = false
var wants_to_interact_tertiary:bool = false

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("player_interact_primary"):
		wants_to_interact_primary = true
	if event.is_action_pressed("player_interact_secondary"):
		wants_to_interact_secondary = true
	if event.is_action_pressed("player_interact_tertiary"):
		wants_to_interact_tertiary = true

func _physics_process(_delta: float) -> void:
	if wants_to_interact_primary or wants_to_interact_secondary or wants_to_interact_tertiary:
		force_raycast_update()
		var collider:Interactable = get_collider() as Interactable
		if collider == null:
			wants_to_interact_primary = false
			wants_to_interact_secondary = false
			wants_to_interact_tertiary = false
			return
		if wants_to_interact_primary:
			collider.interact_primary()
			wants_to_interact_primary = false
		if wants_to_interact_secondary:
			collider.interact_secondary()
			wants_to_interact_secondary = false
		if wants_to_interact_tertiary:
			collider.interact_tertiary()
			wants_to_interact_tertiary = false
