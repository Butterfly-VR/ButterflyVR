extends Node
class_name WorldController

@export var spawn_point:Node3D
@export var chat_box_manager:ChatBoxManager
@export var player_grab_handler:PlayerGrabHandler
@export var avatar_change_handler:AvatarChangeHandler
@export var interactable_handler:InteractableHandler

func _init() -> void:
	GlobalWorldAccess.current_world = self

func _ready() -> void:
	while !(NetworkManager as NetNodeManager).id_ready():
		await get_tree().physics_frame
	if (NetworkManager as NetNodeManager).get_id() == 0:
		chat_box_manager.new_message_sent.connect(log_chat_to_console)

func log_chat_to_console(message:ChatBoxManager.Message) -> void:
	print("Player " + str(message.player) + ": " + message.text)
