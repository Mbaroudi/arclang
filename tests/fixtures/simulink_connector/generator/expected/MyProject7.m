if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject', 'Folder', project_path);
end
cd(project_path);
new_system('MySystem7');
add_block('simulink/Ports & Subsystems/In1', 'MySystem7/in1');
set_param('MySystem7/in1', 'Port', '1');
set_param('MySystem7/in1', 'Position', '[10 60 40 80]');
add_block('simulink/Ports & Subsystems/Out1', 'MySystem7/out1');
set_param('MySystem7/out1', 'Port', '1');
set_param('MySystem7/out1', 'Position', '[100 30 130 50]');
add_block('simulink/Ports & Subsystems/In1', 'MySystem7/in2');
set_param('MySystem7/in2', 'Port', '2');
set_param('MySystem7/in2', 'Position', '[10 10 40 30]');
save_system('MySystem7', fullfile(project_path, 'models/MySystem7.slx'));
file = proj.addFile('models/MySystem7.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
