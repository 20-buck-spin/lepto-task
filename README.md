# LeptoTask 

A cron-like process manager for linux but with additional features: 
1. A web UI to view process and other info 
2. Capture logs from launched processes, if a process is running , view logs in real time
3. Manual start and cancellations of tasks
4. Easily filter data for various tables, the logs table supports full text search 

The project was built using Leptos, a full stack framework for the rust programming language (https://github.com/leptos-rs/leptos)

## CAUTION!
This project is not ready for production environments! 
Currently meant for use in highly trusted environments only.
I am intentionally not providing detailed instructions on how to run this project so that unassuming people are not hurt by 
allowing an RCE engine running rampant on their system! 
The entire project is meant to allow controlled RCE (Remote code Execution).
Authentication has not been implemented yet (might be later if there is enough public interest in the project).
I have tested the build on a linux mint machine (relatively up to date), and an updated arch linux machine.
I highly discourage running the process as root (for obvious safety reasons).

# Motivation
This project started a means of learning the leptos framework

Previously I used plomberry (https://github.com/lucafaggianelli/plombery) to run various python scripts on a schedule. Inspired by that project, and wanting to add some additional features like server side log fts, cancellation of ongoing tasks, and my desire to learn leptos, I became motivated to work on this project. But instead of being limited to python, I thought something that can run other tasks / commands too would be better suited for my needs. There are most probably preexisting tools out there that do this same thing and do it better, but as mentioned earlier, this project started off as a learning experience. 

## Usage
Prebuilt binaries will be made available when (if) the project has at least some security features. I really do not want someone exposing leptotask (in its current state)
over the network without any protection (BAD things can happen, I cannot overstate this enough). 

Steps to schedule first task 
- Clone the repo
- Build/run the program using the leptos cli  https://github.com/leptos-rs/cargo-leptos
- The project requires a DATABASE_URL env variable that points to a sqlite database that it can write to in the format DATABASE_URL="sqlite:./data/app.db?mode=rwc". It will try to look for a .env file in the directory from where the binary was launched. It will then try to find the sqlite db file, run migrations and then launch. 
- Open the project in your browser (localhost:3000 by default)
- In the tasks section, add a new task, and that's it. The task should launch based on the schedule provided. Tasks always run according to schedule, if a previous iteration has not finished, a new instance of the task will be launched anyway. This behaviour might be changed or made configurable in the future.
- If you want leptotask to start on launch, just create a systemd unit (please do not launch it as root, better to create user systemd unit instead)


For pipeline type of work, you can use the pipe operator in the command of the task itself, or write a small script that does the piplining for you

## Todo 
- impl authentication
- Improve UX/UI

## Contributions 
Please create an issue for feature requests 
Contributions are welcome. However, if you want to make any big changes, please open a discussion first.
For small changes (body of a few functions, etc.), feel free to submit a PR

Some things I could use help with are
- A good human made logo (the current one was made using AI, I have no graphic design skills)
- Human made icons (ones used in the project are currently made by AI)
- CSS. I am not good at it, any help to make the project look better would be much appreciated
