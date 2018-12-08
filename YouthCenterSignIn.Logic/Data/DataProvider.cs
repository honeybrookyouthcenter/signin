using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Newtonsoft.Json;

namespace YouthCenterSignIn.Logic.Data
{
    public abstract class DataProvider
    {
        public static DataProvider Current { get; set; }

        #region Json File

        public async Task<T> GetSetting<T>(string key, StorageType storageType)
        {
            string json = "";
            switch (storageType)
            {
                case StorageType.AppSetting:
                    json = GetJsonSetting(key);
                    break;
                case StorageType.LocalFile:
                    json = await GetJsonFileContent(key);
                    break;
            }

            if (string.IsNullOrWhiteSpace(json))
                return default(T);
            else
                return JsonConvert.DeserializeObject<T>(json);
        }

        protected abstract string GetJsonSetting(string key);
        protected abstract Task<string> GetJsonFileContent(string file);

        public async Task SetSetting<T>(string key, T value, StorageType storageType)
        {
            string json = JsonConvert.SerializeObject(value, Formatting.Indented);

            switch (storageType)
            {
                case StorageType.AppSetting:
                    SetJsonSetting(key, json);
                    break;
                case StorageType.LocalFile:
                    await SetJsonFileContent(key, json);
                    break;
            }
        }

        protected abstract void SetJsonSetting(string key, string json);
        protected abstract Task SetJsonFileContent(string file, string json);

        #endregion

        public abstract Task ShowMessage(string message, Exception ex = null);

        public abstract Task<bool> AddPerson(Person person);

        public abstract Task<List<Person>> GetPeople();

        public bool AuthenticateAdmin(string pin)
        {
            if (AdminPin == null)
                AdminPin = "1234";

            return pin == AdminPin;
        }

        string AdminPin
        {
            get => GetSetting<string>(nameof(AdminPin), StorageType.AppSetting).Result; //Waiting is ok because app settings don't need async
            set
            {
                if (value.Length != 4)
                    return;

                var saveTask = SetSetting(nameof(AdminPin), value, StorageType.AppSetting);
            }
        }
    }

    public enum StorageType
    {
        AppSetting,
        LocalFile
    }
}
