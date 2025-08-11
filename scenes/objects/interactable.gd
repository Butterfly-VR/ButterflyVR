extends Area3D
class_name Interactable

signal interacted_primary
signal interacted_secondary
signal interacted_tertiary

@onready var message_handler:InteractableHandler = GlobalWorldAccess.current_world.interactable_handler

func interact_primary() -> void:
	message_handler.send_message(get_path(), 0)
func interact_secondary() -> void:
	message_handler.send_message(get_path(), 1)
func interact_tertiary() -> void:
	message_handler.send_message(get_path(), 2)
