using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic.Tests
{
    public class TestDataProvider : DataProvider
    {
        public override Task ShowMessage(string message = null, Exception ex = null)
        {
            throw new Exception(message, ex);
        }

        #region People

        List<Person> People { get; } = new List<Person>
        {
            new Person(
                id: "ESHJAM",
                firstName: "James",
                lastName: "Esh",
                birthDate: new DateTimeOffset(new DateTime(1999, 2, 23)),
                phoneNumber: "610-883-2281",
                address: "52 evergreen",
                guardian: null),
            new Person(
                id: "BEACHR",
                firstName: "Chris",
                lastName: "Beachy",
                birthDate: new DateTimeOffset(new DateTime(1992, 2, 18)),
                phoneNumber: "(123) 234-3456",
                address: "honey brook",
                guardian: null),
            new Person(
                id: "GLIDAN",
                firstName: "Daniel",
                lastName: "Glick",
                birthDate: new DateTimeOffset(new DateTime(1999, 7, 12)),
                phoneNumber: "9878767654",
                address: "gap pa",
                guardian: null),
            new Person(
                id: "PETMER",
                firstName: "Merv",
                lastName: "Petershiem",
                birthDate: new DateTimeOffset(new DateTime(1975, 12, 3)),
                phoneNumber: "717-626-5353",
                address: "youth center",
                guardian: null),
        };

        public override Task<bool> AddPerson(Person person)
        {
            People.Add(person);

            return Task.FromResult(true);
        }

        public override Task<List<Person>> GetPeople()
        {
            return Task.FromResult(People);
        }

        #endregion

        #region Json

        #region Files

        Dictionary<string, string> JsonFiles { get; } = new Dictionary<string, string>();

        protected override Task<string> GetJsonFileContent(string file)
        {
            if (JsonFiles.TryGetValue(file, out string json))
                return Task.FromResult(json);

            return Task.FromResult("");
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
    }
}
