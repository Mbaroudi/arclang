if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject20');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject20', 'Folder', project_path);
end
cd(project_path);
MySignalInSeconds = Simulink.Signal;
MySignalInSeconds.DataType = 'int64';
MySignalInSeconds.InitialValue = '11.23';
MySignalInSeconds.Unit = 's';
MySignalInSeconds.Description = 'ModelerId:originSignalInSeconds';
assignin('base', 'MySignalInSeconds', MySignalInSeconds);
MySignalInTons = Simulink.Signal;
MySignalInTons.DataType = 'int8';
MySignalInTons.InitialValue = '2';
MySignalInTons.Unit = 't';
MySignalInTons.Description = 'ModelerId:originSignalInTons';
assignin('base', 'MySignalInTons', MySignalInTons);
if isfile('data/MyProject20_dict.sldd')
	MyProject20_dict = Simulink.data.dictionary.open('data/MyProject20_dict.sldd');
else
	MyProject20_dict = Simulink.data.dictionary.create('data/MyProject20_dict.sldd');
end
importFromBaseWorkspace(MyProject20_dict, 'varList', {'MySignalInSeconds', 'MySignalInTons'}, 'existingVarsAction','overwrite');
saveChanges(MyProject20_dict);
clear MyProject20_dict;
file = proj.addFile('data/MyProject20_dict.sldd');
evalin('base', "clear 'MySignalInSeconds' 'MySignalInTons'");
new_system('mySys');
set_param('mySys', 'DataDictionary', 'MyProject20_dict.sldd');
try
	Simulink.BlockDiagram.arrangeSystem('mySys');
catch
end
save_system('mySys', fullfile(project_path, 'models/mySys.slx'));
file = proj.addFile('models/mySys.slx');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
