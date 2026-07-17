# EROP (Elden Ring QWOP)

Elden Ring mod changes the movement controls to a nearly jank-for-jank implementation of the classic Bennett Foddy game [QWOP](https://www.foddy.net/legacy/Athletics.html).

<img width="1020" height="400" alt="Image" src="https://github.com/user-attachments/assets/ca999c10-ab7e-462f-b078-c8954a9b894e" />

## Installation

1. [Install me3](https://github.com/garyttierney/me3/releases/latest)
2. Download the mod and extract it somehwere
3. Double-click `erop.me3` from Windows Explorer to start the mod

Works with Elden Ring patch 1.16.2. Not tested for compatibility with any other mods.

## Usage

- Press Q/W to move your thighs, and O/P to move your calves.
- Press X to toggle normal controls on or off.
- Turn the camera or lock onto an enemy to change the direction you walk in.

The goal is to walk without falling on the ground. Falling over deals damage and resets your position.

<img width="1020" height="400" alt="Image" src="https://github.com/user-attachments/assets/b033797a-0e94-461d-9193-4a26f925e450" />

All other forms of movement are disabled while QWOP controls are on. Attacking, casting spells, using items, and jumping are still allowed, but walking, rolling, crouching, backstepping, and riding Torrent are not.

Tip: You can change the keybindings to whatever you want in the game's keyboard settings. I recommend finding something that you can comfortably press with one hand, so your right hand can stay on the mouse.

### Can you actually beat the game like this?

Probably. I think so. I don't expect anyone to, but it should be possible with a lot of patience.

If I were trying to beat the game with this mod, I'd probably allow toggling normal controls for overworld travel and mandatory jumps. I'd probably also ban ranged attacks.

### Advanced jank

It's probably not important to know all the subtle quirks, but in case you're attempting to actually beat the game (please don't) I feel obligated to provide some info that might be helpful:

1. The QWOP physics assumes you're standing on flat ground. You may notice your legs visually clipping through rocks and hills in the game. Just note that for physics purposes, variations in elevation are ignored.
2. You're considered to have fallen when your head or torso makes contact with the surface level. Surface level is measured as if there were a completely flat plane immediately below the player.
3. Moving forward as part of an attack animation is allowed, but you will return to your original position as the animation ends. This is to avoid letting you cheese forward movement by attacking, without nerfing melee movesets that rely on thrusting forward for range.
4. Doing a split allows you to duck under certain attacks.

## Credits

- Mod by Tom Clark
- Inspired by the work of [Bennett Foddy](https://foddy.net/)
- Uses fromsoftware-rs by vswarte and others. Also made possible by indura's research into keybindings, and Dasaav's research into runtime skeleton modification
