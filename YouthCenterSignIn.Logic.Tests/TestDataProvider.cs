using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic.Tests
{
    public class TestDataProvider : DataProvider
    {
        #region People

        List<Person> People { get; } = new List<Person>
        {
            new Person(id: "ESHJAM", firstName: "James", lastName: "Esh"),
            new Person(id: "BEACHR", firstName: "Chris", lastName: "Beachy"),
            new Person(id: "GLIDAN", firstName: "Daniel", lastName: "Glick"),
            new Person(id: "PETMER", firstName: "Merv", lastName: "Petershiem"),
        };

        public override Task AddPerson(Person person)
        {
            People.Add(person);

            return Task.CompletedTask;
        }

        public override Task<List<Person>> GetPeople()
        {
            return DoAsyncInvoke(() => People);
        }

        #endregion

        #region Json

        #region Files

        Dictionary<string, string> JsonFiles { get; } = new Dictionary<string, string>();

        protected override Task<string> GetJsonFileContent(string file)
        {
            return DoAsyncInvoke(() =>
            {
                if (JsonFiles.TryGetValue(file, out string json))
                    return json;

                return "";
            });
        }

        protected override Task SetJsonFileContent(string file, string json)
        {
            if (JsonFiles.ContainsKey(file))
                JsonSettings[file] = json;
            else
                JsonSettings.Add(file, json);

            return Task.CompletedTask;
        }

        #endregion

        #region Settings

        Dictionary<string, string> JsonSettings { get; } = new Dictionary<string, string>();

        protected override string GetJsonSetting(string key)
        {
            if (JsonSettings.TryGetValue(key, out string json))
                return json;

            return "";
        }

        protected override void SetJsonSetting(string key, string json)
        {
            if (JsonSettings.ContainsKey(key))
                JsonSettings[key] = json;
            else
                JsonSettings.Add(key, json);
        }

        #endregion 

        #endregion

        public Task RunningTask { get; private set; }
        Task<T> DoAsyncInvoke<T>(Func<T> action)
        {
            if (RunningTask == null || RunningTask.IsCompleted)
                RunningTask = new Task<T>(action);
            else
                RunningTask = RunningTask.ContinueWith((t) => new Task<T>(action));

            return (Task<T>)RunningTask;
        }
    }
}
