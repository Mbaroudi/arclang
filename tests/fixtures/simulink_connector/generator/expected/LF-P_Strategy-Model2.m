if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'capella_mls_ref_project');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'capella_mls_ref_project', 'Folder', project_path);
end
cd(project_path);
new_system('LF_C_Sys');
add_block('simulink/Ports & Subsystems/In1', 'LF_C_Sys/FIP_1');
set_param('LF_C_Sys/FIP_1', 'Port', '1');
add_block('simulink/Ports & Subsystems/In1', 'LF_C_Sys/FIP_2');
set_param('LF_C_Sys/FIP_2', 'Port', '2');
add_block('simulink/Ports & Subsystems/In1', 'LF_C_Sys/FIP_3');
set_param('LF_C_Sys/FIP_3', 'Port', '3');
t_Block = add_block('simulink/Sinks/Terminator', 'LF_C_Sys/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('LF_C_Sys/FIP_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('LF_C_Sys', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
t_Block = add_block('simulink/Sinks/Terminator', 'LF_C_Sys/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('LF_C_Sys/FIP_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('LF_C_Sys', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
t_Block = add_block('simulink/Sinks/Terminator', 'LF_C_Sys/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('LF_C_Sys/FIP_3', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('LF_C_Sys', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('LF_C_Sys');
catch
end
save_system('LF_C_Sys', fullfile(project_path, 'models/LF_C_Sys.slx'));
file = proj.addFile('models/LF_C_Sys.slx');
new_system('LF_P');
add_block('simulink/Ports & Subsystems/In1', 'LF_P/FIP_1__LF_C_Sys_');
set_param('LF_P/FIP_1__LF_C_Sys_', 'Port', '1');
add_block('simulink/Ports & Subsystems/Enable', 'LF_P/S1_1__S1_2__S1_3');
set_param('LF_P/S1_1__S1_2__S1_3', 'Description', 'S1/1, S1/2, S1/3');
add_block('CapellaElements/Subsystem/Function', 'LF_P/LF_C_Sub_Sys');
CapSLBridge.internal.cleanSubsystem('LF_P/LF_C_Sub_Sys');
add_block('simulink/Ports & Subsystems/In1', 'LF_P/LF_C_Sub_Sys/FIP_1__LF_C_Sys_');
set_param('LF_P/LF_C_Sub_Sys/FIP_1__LF_C_Sys_', 'Port', '1');
add_block('CapellaElements/ModelRef/Model_Function', 'LF_P/LF_C_Sub_Sys/ref_to_LF_C_Sub_Sys');
set_param('LF_P/LF_C_Sub_Sys/ref_to_LF_C_Sub_Sys', 'ModelFile', 'LF_C_Sys')
handles = get_param('LF_P/LF_C_Sub_Sys/FIP_1__LF_C_Sys_', 'PortHandles');
source = handles.Outport(1);
handles = get_param('LF_P/LF_C_Sub_Sys/ref_to_LF_C_Sub_Sys', 'PortHandles');
target = handles.Inport(2);
line = add_line('LF_P/LF_C_Sub_Sys', source, target, 'autorouting','smart');
set_param(line, 'Name', 'a');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('LF_P/LF_C_Sub_Sys');
catch
end
handles = get_param('LF_P/FIP_1__LF_C_Sys_', 'PortHandles');
source = handles.Outport(1);
handles = get_param('LF_P/LF_C_Sub_Sys', 'PortHandles');
target = handles.Inport(1);
line = add_line('LF_P', source, target, 'autorouting','smart');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('LF_P');
catch
end
save_system('LF_P', fullfile(project_path, 'models/LF_P.slx'));
file = proj.addFile('models/LF_P.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
