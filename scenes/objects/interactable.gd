extends Area3D
class_name Interactable
signal interacted_primary
signal interacted_secondary
signal interacted_tertiary

func _physics_process(_delta: float) -> void:
	if (NetworkManager as NetNodeManager).has_message() and (NetworkManager as NetNodeManager).get_message_type() == 8:
		var interact_type:int = (NetworkManager as NetNodeManager).peek_message()[0]
		var path:String = (NetworkManager as NetNodeManager).peek_message()[1]
		if path == str(get_path()):
			match interact_type:
				0: interacted_primary.emit()
				1: interacted_secondary.emit()
				2: interacted_tertiary.emit()
				_: push_warning("unhandled interaction type")
			(NetworkManager as NetNodeManager).pop_message()

func interact_primary() -> void:
	(NetworkManager as NetNodeManager).trigger_interaction((NetworkManager as NetNodeManager).get_id(), 0, str(get_path()))
func interact_secondary() -> void:
	(NetworkManager as NetNodeManager).trigger_interaction((NetworkManager as NetNodeManager).get_id(), 1, str(get_path()))
func interact_tertiary() -> void:
	(NetworkManager as NetNodeManager).trigger_interaction((NetworkManager as NetNodeManager).get_id(), 2, str(get_path()))
