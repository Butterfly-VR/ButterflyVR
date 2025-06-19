extends Label3D

var fading:bool = false

func _ready() -> void:
	@warning_ignore("unsafe_property_access", "unsafe_method_access")
	GlobalWorldAccess.current_world.get_child(0).get_child(2).new_message_sent.connect(on_message)

func on_message(message:ChatBoxManager.Message) -> void:
	@warning_ignore("unsafe_property_access")
	if message.player == get_parent().id:
		text = message.text
		transparency = 0
		await get_tree().create_timer(3).timeout
		fading = true

func _process(delta: float) -> void:
	if fading:
		transparency += 0.5 * delta
		if transparency > 0.99:
			transparency = 1
			fading = false
